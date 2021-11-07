import Vue from 'vue'
import App from './App.vue'
import router from './router'
import lodash from 'lodash'
import './theme.scss'

Vue.config.productionTip = false
Vue.prototype._ = window._ = lodash

new Vue({
  router,
  render: h => h(App)
}).$mount('#app')
