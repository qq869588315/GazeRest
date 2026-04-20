import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import enUS from './en-US'
import zhCN from './zh-CN'

void i18n.use(initReactI18next).init({
  lng: 'zh-CN',
  fallbackLng: 'zh-CN',
  interpolation: {
    escapeValue: false,
  },
  resources: {
    'zh-CN': { translation: zhCN },
    'en-US': { translation: enUS },
  },
})

export default i18n
