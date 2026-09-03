// Service Worker for Revisited IPIP-NEO - Atomic Cache-First with Background Network Revalidation
const CACHE_NAME = 'ipip-neo-cache-v20260904';

// Static assets to pre-cache on install
const PRECACHE_ASSETS = [
  './',
  './index.html',
  './manifest.json',
  './favicon.ico'
];

// 1. Pre-cache on install and activate immediately
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(PRECACHE_ASSETS)).then(() => self.skipWaiting())
  );
});

// 2. Purge all legacy caches on activation
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

// 3. Fetch router tailored for atomic immutable serverless deployments
self.addEventListener('fetch', (event) => {
  // Only handle local same-origin GET requests
  if (event.request.method !== 'GET' || !event.request.url.startsWith(self.location.origin)) {
    return;
  }

  const url = new URL(event.request.url);

  // Bypass third-party analytics or beacon endpoints
  if (url.hostname.includes('cloudflareinsights.com') || url.hostname.includes('google-analytics.com')) {
    return;
  }

  // Never cache the service worker script itself so browser can check for updates in background
  if (url.pathname.endsWith('/sw.js')) {
    event.respondWith(fetch(event.request));
    return;
  }

  const isNavigation = event.request.mode === 'navigate' || event.request.destination === 'document' || url.pathname.endsWith('.html') || url.pathname === '/';

  event.respondWith(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.match(event.request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }
        return fetch(event.request).then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            cache.put(event.request, networkResponse.clone());
          }
          return networkResponse;
        }).catch((err) => {
          if (isNavigation) {
            return cache.match('./index.html') || cache.match('./');
          }
          throw err;
        });
      });
    })
  );
});
