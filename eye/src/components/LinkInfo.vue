<template lang="html">
  <div class="link">
    <div class="link-name">{{ source || link.out }}</div>
    <div class="link-info">th: {{ node.threshold }}</div>
    <div class="link-info">s: {{ node.score.toFixed(2) }}</div>
    <template v-if="link">
      <div class="link-info">w: {{ link.weight.toFixed(2) }}</div>
      <div class="link-info">d: {{ link.delay }}</div>
    </template>
  </div>
</template>

<script>
export default {
  props: ['source', 'link', 'net'],
  computed: {
    node () {
      return this.net.nodes[this.source || this.link.out]
    }
  }
}
</script>

<style lang="scss">
.link {
  // border: 1px solid #000;
  text-align: left;
  padding: 5px;
  min-width: 10%;
  transition: all .1s;
  cursor: pointer;
  &:hover {
    background: #111;
    // color: #000;
  }
  &:not(:hover) {
    opacity: .5;
  }
  .link-name {
    color: #fa1;
    white-space: nowrap;
    font-size: 0.8rem;
  }
  .link-info {
    white-space: nowrap;
    // font-size: 0.8rem;
  }
}
</style>
