// Service Worker for Revisited IPIP-NEO (TGA) Offline Mode
const CACHE_NAME = 'ipip-neo-tga-cache-v1';

// Install event: Pre-cache core shell
self.addEventListener('install', (event) => {
  self.skipWaiting();
});

// Activate event: Clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      );
    }).then(() => self.clients.claim())
  );
});

// Fetch event: Cache-First strategy with Network fallback for offline functionality
self.addEventListener('fetch', (event) => {
  // Only handle GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  // Bypass analytics beacons
  const url = new URL(event.request.url);
  const isAllowedHostOrSubdomain = (hostname, domain) => {
    return hostname === domain || hostname.endsWith(`.${domain}`);
  };
  if (
    isAllowedHostOrSubdomain(url.hostname, 'cloudflareinsights.com') ||
    isAllowedHostOrSubdomain(url.hostname, 'google-analytics.com')
  ) {
    return;
  }

  event.respondWith(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.match(event.request).then((cachedResponse) => {
        // Fetch from network in background or when not cached
        const fetchPromise = fetch(event.request)
          .then((networkResponse) => {
            if (networkResponse && networkResponse.status === 200) {
              cache.put(event.request, networkResponse.clone());
            }
            return networkResponse;
          })
          .catch(() => {
            // When offline and not in cache, fallback to root index.html if navigating
            if (event.request.mode === 'navigate') {
              return cache.match('./') || cache.match('./index.html') || cache.match('index.html');
            }
          });

        return cachedResponse || fetchPromise;
      });
    })
  );
});
