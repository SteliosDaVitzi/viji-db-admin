<script setup lang="ts">
const { messages, dismiss } = useToast()
const target = shallowRef<HTMLElement | string>('body')
const region = useTemplateRef<HTMLElement>('region')
const timers = new Map<number, ReturnType<typeof setTimeout>>()
const paused = new Set<number>()
let observer: MutationObserver | undefined

function pause(id: number) {
  paused.add(id)
  clearTimeout(timers.get(id))
  timers.delete(id)
}
function resume(id: number) {
  pause(id)
  paused.delete(id)
  timers.set(
    id,
    setTimeout(() => dismiss(id), 6500),
  )
}
watch(
  messages,
  async (items) => {
    for (const id of timers.keys()) if (!items.some((item) => item.id === id)) pause(id)
    for (const id of paused) if (!items.some((item) => item.id === id)) paused.delete(id)
    for (const item of items) if (!timers.has(item.id) && !paused.has(item.id)) resume(item.id)
    await nextTick()
    if (items.length && !region.value?.matches(':popover-open')) region.value?.showPopover()
  },
  { immediate: true },
)
onMounted(() => {
  // Put feedback inside the active modal's focus scope and above its backdrop.
  function updateTarget() {
    const dialogs = document.querySelectorAll<HTMLDialogElement>('dialog[open]')
    const next = dialogs[dialogs.length - 1] || 'body'
    if (target.value !== next) target.value = next
  }
  updateTarget()
  observer = new MutationObserver(updateTarget)
  observer.observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ['open'],
  })
})
watch(target, async () => {
  await nextTick()
  if (messages.value.length && !region.value?.matches(':popover-open')) region.value?.showPopover()
})
onBeforeUnmount(() => {
  observer?.disconnect()
  for (const id of timers.keys()) pause(id)
})
</script>

<template>
  <Teleport :to="target">
    <div ref="region" class="toast-region" popover="manual" aria-label="Notifications">
      <TransitionGroup name="toast" tag="div" class="toast-stack">
        <div
          v-for="item in messages"
          :key="item.id"
          class="toast-message"
          :class="`toast-${item.tone}`"
          :role="item.tone === 'error' ? 'alert' : 'status'"
          aria-atomic="true"
          @mouseenter="pause(item.id)"
          @mouseleave="resume(item.id)"
          @focusin="pause(item.id)"
          @focusout="resume(item.id)"
        >
          <AppIcon :name="item.tone === 'success' ? 'check' : 'database'" :size="18" />
          <span>{{ item.message }}</span>
          <button
            class="icon-button"
            type="button"
            aria-label="Close notification"
            @click="dismiss(item.id)"
          >
            <AppIcon name="close" :size="15" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
