/// <reference no-default-lib="true" />
/// <reference lib="WebWorker" />
/// <reference lib="esnext" />

const sw = self as ServiceWorkerGlobalScope & typeof globalThis

sw.addEventListener('fetch', event => {
  event.respondWith(
    caches
      .match(event.request)
      .then(res => {
        return res || fetch(event.request)
      })
  )
})

