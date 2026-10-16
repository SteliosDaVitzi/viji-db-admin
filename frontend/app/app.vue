<template>
  <div class="app-shell">
    <NuxtRouteAnnouncer />
    <NuxtPage :keepalive="{ include: ['index'] }" />
    <PrivacyControls />
    <ClientOnly><ToastHost /></ClientOnly>
  </div>
</template>
