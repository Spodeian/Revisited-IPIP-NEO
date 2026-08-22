// Service Worker for Revisited IPIP-NEO (TGA) - Network-First Offline Cache Strategy
const CACHE_NAME = 'ipip-neo-tga-cache-v2';

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

// Fetch event: Network-First strategy with Cache fallback
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
    // isAllowedHostOrSubdomain(url.hostname, 'google-analytics.com')
  ) {
    return;
  }

  event.respondWith(
    caches.open(CACHE_NAME).then((cache) => {
      // Network-First: Always try to fetch from the network first
      return fetch(event.request)
        .then((networkResponse) => {
          // If successful, update the cache and return the fresh resource
          if (networkResponse && networkResponse.status === 200) {
            cache.put(event.request, networkResponse.clone());
          }
          return networkResponse;
        })
        .catch(() => {
          // Cache fallback: If offline/network fails, load from local cache
          return cache.match(event.request).then((cachedResponse) => {
            if (cachedResponse) {
              return cachedResponse;
            }
            // If offline, not in cache, and navigating, fallback to root index
            if (event.request.mode === 'navigate') {
              return cache.match('./') || cache.match('./index.html') || cache.match('index.html');
            }
          });
        });
    })
  );
});
