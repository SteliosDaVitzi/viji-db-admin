export default defineEventHandler(async (event) => {
  // Browser mutations must come from this application, including disconnect.
  if (!['GET', 'HEAD'].includes(event.method)) {
    const origin = getHeader(event, 'origin')
    if (!origin || origin !== getRequestURL(event).origin) {
      throw createError({ statusCode: 403, statusMessage: 'Request origin rejected.' })
    }
  }
  setResponseHeader(event, 'Cache-Control', 'no-store')
  const config = useRuntimeConfig(event)
  const path = getRequestURL(event).pathname + getRequestURL(event).search
  return proxyRequest(event, `${config.apiBase}${path}`, {
    headers: { host: new URL(config.apiBase).host },
  })
})
