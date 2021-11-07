<template>
  <div class="home">
    <div>
      <div class="chart" v-if="(i=0) || (ch = (charts[i] && charts[i][0]))">
        <div>Allele per gene (historical)</div>
        <canvas ref="c-0" width="600px" height="400px"/>
        <br>
        <span class="blue">{{ ch.length }}</span>
        -
        <span class="green">{{ ch.reduce((max, i) => Math.max(max, i), -Infinity) }}</span>
        -
        <span class="red">{{ ch.reduce((max, i) => Math.min(max, i), Infinity) }}</span>
      </div>

      <div class="chart" v-if="(i=1) && (ch = (charts[i] && charts[i][0]))">
        <div>Avg Score</div>
        <canvas ref="c-1" width="600px" height="400px"/>
        <br>
        <span class="blue">{{ ch.length }}</span>
        -
        <span class="green">{{ ch.reduce((max, i) => Math.max(max, i), -Infinity) }}</span>
        -
        <span class="red">{{ ch.reduce((max, i) => Math.min(max, i), Infinity) }}</span>
      </div>

      <div class="chart" v-if="(i=2) && (ch = (charts[i] && charts[i][0]))">
        <div>Alleles per gene</div>
        <canvas ref="c-2" width="600px" height="400px"/>
        <br>
        <span class="blue">{{ ch.length }}</span>
        -
        <span class="green">{{ ch.reduce((max, i) => Math.max(max, i), -Infinity) }}</span>
        -
        <span class="red">{{ ch.reduce((max, i) => Math.min(max, i), Infinity) }}</span>
      </div>

      <div class="chart" v-if="(i=3) && (ch = (charts[i] && charts[i][0]))">
        <div>Mutation rate:</div>
        <canvas ref="c-3" width="600px" height="400px"/>
        <br>
        <span class="blue">{{ ch.length }}</span>
        -
        <span class="green">{{ ch.reduce((max, i) => Math.max(max, i), -Infinity) }}</span>
        -
        <span class="red">{{ ch.reduce((max, i) => Math.min(max, i), Infinity) }}</span>
      </div>

      <!-- <pre>{{ ga() && ga().genes['!in 10-10']}}</pre> -->
    </div>
  </div>
</template>

<script>
export default {
  name: 'home',
  components: {
  },
  data () {
    return {
      ctx: null,
      info: {
        genes: {
          asdf: {
            received_alleles: 10,
            alleles: [1,2],
          },
          cdsa: {
            received_alleles: 10,
            alleles: [1,2],
          },
          dsa: {
            received_alleles: 33,
            alleles: [1,2,4],
          },
        }
      },
      raw: null,
      graphs: null,
      charts: [],
      debug: null,
    }
  },

  computed: {
  },

  mounted () {
    this.update()
  },
  beforeDestroy () {
    clearTimeout(this._to)
  },

  methods: {
    ga () {
      console.log('compute ga')
      return this._raw && this._raw.main_net.gas[this._raw.main_net.gas.length-1]
    },
    updatedCharts () {
      console.log('compute charts')
      let charts = []
      let rand = []
      while (rand.length < 900) rand.push(Math.random())
      // charts.push(rand)
      let g, max
      let genes = Object.values(this.ga().genes)

      g = genes
        .map(g => g.received_alleles)
        .sort((a, b) => a-b)
        // .filter(i => !(i%3))
      max = g.reduce((max, v) => Math.max(max, v), -Infinity)
      charts.push([g, 0, max])

      g = genes
        .map(g => _.meanBy(_.values(g.alleles), a => a.avg_score))
        .sort((a, b) => a-b)
        // .filter(i => !(i%3))
      max = g.reduce((max, v) => Math.max(max, v), -Infinity)
      charts.push([g, max-100, max])

      g = genes
        .map(g => Object.values(g.alleles).length)
        .sort((a, b) => a-b)
        // .filter(i => !(i%3))
      max = g.reduce((max, v) => Math.max(max, v), -Infinity)
      charts.push([g, 0, max])

            // g = Object.values(this._raw.main_net.gas[0].leaderboard[0].nodes)
            // .map(n => n.score)
            //   .sort((a, b) => a-b)
            //   // .filter(i => !(i%3))
            // max = g.reduce((max, v) => Math.max(max, v), -Infinity)
            // var min = g.reduce((min, v) => Math.min(min, v), Infinity)
            // charts.push([g, min, max + 0.1])

      g = genes
        .map(g => g.alleles[Object.keys(g.alleles)[0]].node.links.length)
        .sort((a, b) => a-b)
        // .filter(i => !(i%3))
      max = g.reduce((max, v) => Math.max(max, v), -Infinity)
      charts.push([g, 0, max])


      this.charts = charts
      return charts
    },
    update () {
      console.log('download', this.$route.query.url)
      fetch('http://'+(this.$route.query.url || '127.0.0.1')+':8090/stats')
      .then(res => {
        console.log('got raw')
        return res.json()
      })
      .then(stats => {
        console.log('got json')
        this._raw = stats
        console.log('saved')
        this.updatedCharts()
        setTimeout(() => {
          console.log('draw')
          this.charts.forEach((data, chart_i) => {
            console.log(`c-${chart_i}`)
            let cvs = this.$refs[`c-${chart_i}`]
            let ctx = cvs.getContext('2d')
            let w = cvs.width
            let h = cvs.height
            ctx.clearRect(0, 0, w, h)
            ctx.strokeWidth = 0
            // console.log(data)
            // ctx.fillStyle = 'rgb('+[0, parseInt(x/w*255), parseInt((w-x)/w*255)].join(',')+')'

            var grd = ctx.createLinearGradient(0, 0, w, 0);
            grd.addColorStop(0, "#f00");
            // grd.addColorStop(0.5, 'rgb(255,255,0)');
            grd.addColorStop(0.5, '#ff0');
            grd.addColorStop(1, "#0f0");
            // ctx.fillStyle = 'rgba(255,155,10,.5)'
            ctx.fillStyle = grd
            ctx.beginPath()
            ctx.moveTo(0, h)
            data[0].forEach((y, x) => {
              y = (y-data[1])/(data[2] - data[1])
              // ctx.fillStyle = 'rgb('+[0, parseInt(x/w*255), parseInt((w-x)/w*255)].join(',')+')'
              ctx.lineTo(x/(data[0].length-1)*w, h - h*y)
              // ctx.fillRect(x, h - h*y/max, 1, h)
            })
            ctx.lineTo(w, h)
            ctx.closePath()

            ctx.fill()

          })
        })
      })
      .finally(() => {
        this._to = setTimeout(() => this.update(), 5000)
      })
    }
  }
}
</script>

<style lang="scss">
.chart {
  height: 43vh;
  width: 45vw;
  padding: 20px;
  float: left;
  canvas {
    box-shadow: 0 0 30px rgba(255,255,255,.1);
    max-height: 90%;
    max-width: 100%;
  }
}
</style>
