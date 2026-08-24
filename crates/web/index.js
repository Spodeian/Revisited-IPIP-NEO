// Register Service Worker for offline PWA capabilities
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register('./sw.js')
      .then((reg) => console.log('ServiceWorker active on scope:', reg.scope))
      .catch((err) => console.log('ServiceWorker registration deferred:', err));
  });
}
