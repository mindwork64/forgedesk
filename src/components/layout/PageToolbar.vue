<template>
  <div class="fd-page-toolbar" data-testid="page-toolbar">
    <div class="fd-page-toolbar__left">
      <slot name="left" />
    </div>
    <div class="fd-page-toolbar__right">
      <slot name="right" />
    </div>
  </div>
</template>

<style scoped>
.fd-page-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fd-space-3);
  height: var(--fd-toolbar-height);
  padding: 0 var(--fd-content-pad);
  border-bottom: 1px solid var(--fd-border-subtle);
}

.fd-page-toolbar__left,
.fd-page-toolbar__right {
  display: flex;
  align-items: center;
  gap: var(--fd-space-2);
  min-width: 0;
}

.fd-page-toolbar__right {
  justify-content: flex-end;
}
</style>
