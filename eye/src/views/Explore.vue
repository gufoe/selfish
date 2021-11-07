<template>
  <div class="about">
    <h1>Node explorer</h1>
    <button @click="refresh()">Refresh</button>
    <!-- <NodeExplorer/> -->
    <div v-if="!net">
      Loading...
    </div>
    <template v-else>
      <h2>Score: {{ net.score.toFixed(3) }}</h2>
      <h2>Max: {{ _.max(_.map(net.nodes, n => n.score)) }}</h2>
      <h2>Min: {{ _.min(_.map(net.nodes, n => n.score)) }}</h2>
      <div v-if="_node">
        <NodeExplorer v-model="_nname" :net="net"/>
      </div>
      <div v-else>
        <div style="max-width: 500px; margin: 0 auto; text-align: left">
          <div v-for="(node, node_name) in net.nodes" @click="_nname = node_name">
            {{ node.links.length }} - {{ node.score.toFixed(2) }} - {{ node_name }}
          </div>
        </div>
      </div>
    </template>
  </div>
</template>


<script type="text/javascript">
import NodeExplorer from '@/components/NodeExplorer'

export default {
  components: {
    NodeExplorer,
  },

  data () {
    return {
      net: null,
    }
  },
  computed: {
    _nname: {
      get () {
        return this.$route.params.node
      },
      set (name) {
        this.$router.push({
          params: { node: name }
        })
      }
    },
    _node () {
      return this.net && this.net.nodes[this._nname]
    },
  },

  mounted () {
    this.refresh()
  },
  methods: {
    refresh () {
      this.net = null
      fetch('http://'+(this.$route.query.url || '127.0.0.1')+':8090/stats')
      .then(res => {
        console.log('got raw')
        return res.json()
      })
      .then(stats => {
        this.net = stats.main_net.gas[stats.main_net.gas.length-1].leaderboard[0]
      })
    },
  },

}
</script>
