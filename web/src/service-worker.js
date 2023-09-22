/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />
import { build, files, version } from '$service-worker';

const sw = /** @type {ServiceWorkerGlobalScope} */ (/** @type {unknown} */ (self));

const CACHE = `cache-${version}`;
const ASSETS = [...build, ...files];

sw.addEventListener('install', event => {
  async function addFilesToCache() {
    const cache = await caches.open(CACHE);
    await cache.addAll(ASSETS);
  }

  event.waitUntil(addFilesToCache());
});

sw.addEventListener('activate', event => {
  async function deleteOldCaches() {
    for (const key of await caches.keys()) {
      if (key !== CACHE) await caches.delete(key);
    }
  }

  event.waitUntil(deleteOldCaches());
});

sw.addEventListener('fetch', event => {
  // ignore non-GET requests
  if (event.request.method !== 'GET') return;

  const respond = async () => {
    const url = new URL(event.request.url);
    const cache = await caches.open(CACHE);

    // `build`/`files` can always be served from the cache
    if (ASSETS.includes(url.pathname)) {
      return cache.match(url.pathname);
    }

    try {
      return caches.match(event.request)
    } catch (e) {
      const response = await fetch(event.request);

      if (response.status === 200) {
        cache.put(event.request, response.clone());
      }

      return response;
    }
  }

  event.respondWith(respond())
})
