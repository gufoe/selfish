<template lang="html">
  <div class="node-explorer">
    <LinkInfo
    :net="net"
    :source="nname"/>
    <hr>
    <h2>Output</h2>
    <div class="nodes">
      <LinkInfo
      v-for="link in node.links"
      :net="net"
      :link="link"
      @click.native="$emit('input', link.out)"/>
    </div>
    <hr>
    <h2>Inputs</h2>
    <input type="text" v-model="search">
    <div class="nodes">
      <LinkInfo
      v-for="(link, source) in inputs" v-if="!search || source.match(search)"
      :net="net"
      :source="source" :link="link"
      @click.native="$emit('input', source)"/>
    </div>
  </div>
</template>

<script>
import LinkInfo from '@/components/LinkInfo'

export default {
  components: {
    LinkInfo,
  },
  name: 'NodeExplorer',
  props: ['net', 'value'],

  data () {
    return {
      search: '',
    }
  },

  computed: {
    node () {
      return this.net.nodes[this.value]
    },
    nname () {
      return this.value
    },
    inputs() {
      let ret = {}
      for (var name in this.net.nodes) {
        let n = this.net.nodes[name]
        let l = n.links.find(l => l.out == this.nname)
        if (l) ret[name] = l
      }
      return ret
    }
  }
}
</script>

<style lang="scss">
.node-explorer {
  max-width: 1000px;
  margin: 0 auto;
  .nodes {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
  }
}
</style>
