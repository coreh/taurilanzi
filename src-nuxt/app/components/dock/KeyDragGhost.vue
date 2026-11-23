<script setup lang="ts">
import { Plus } from '@lucide/vue'

const { drag } = useKeyDrag()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="drag?.image"
      class="pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-1/2"
      :style="{ left: `${drag.x}px`, top: `${drag.y}px` }"
    >
      <img
        :src="drag.image"
        alt=""
        class="opacity-90 shadow-2xl ring-2 ring-sky-400"
        :class="drag.wide ? 'h-14 w-32 rounded-[4%/9%]' : 'size-14 rounded-[9%]'"
      >
      <span
        v-if="drag.copy"
        class="absolute -right-2 -bottom-2 flex size-5 items-center justify-center rounded-full bg-emerald-500 text-white shadow ring-2 ring-black"
      >
        <Plus class="size-3.5" :stroke-width="3" />
      </span>
    </div>
  </Teleport>
</template>
