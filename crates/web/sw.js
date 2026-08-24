// Service Worker for Revisited IPIP-NEO (TGA) - Stale-While-Revalidate Caching Strategy
const CACHE_NAME = 'ipip-neo-tga-cache-v2';

// Essential static assets to pre-cache on install
const STATIC_ASSETS = [
  './',
  './index.html',
  './manifest.json'
];

// Pre-cache static shell roots
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(STATIC_ASSETS))
  );
  self.skipWaiting();
});

// Clean up previous cache versions
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

// Stale-While-Revalidate fetch handler
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET' || !event.request.url.startsWith(self.location.origin)) {
    return;
  }

  event.respondWith(
    caches.open(CACHE_NAME).then((cache) =>
      cache.match(event.request).then((cachedResponse) => {
        const fetchPromise = fetch(event.request)
          .then((networkResponse) => {
            if (networkResponse && networkResponse.status === 200) {
              cache.put(event.request, networkResponse.clone());
            }
            return networkResponse;
          })
          .catch((err) => {
            if (cachedResponse) return cachedResponse;
            throw err;
          });

        return cachedResponse || fetchPromise;
      }).catch(() => {
        if (event.request.mode === 'navigate') {
          return cache.match('./index.html') || cache.match('./');
        }
      })
    )
  );
});
