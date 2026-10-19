import DefaultTheme from 'vitepress/theme'
import { h } from 'vue'
import HomeVideo from './components/HomeVideo.vue'
import './custom.css'

export default {
    extends: DefaultTheme,
    Layout: () => {
        return h(DefaultTheme.Layout, null, {
            'home-features-before': () => h(HomeVideo)
        })
    }
}
