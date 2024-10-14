import { StreamLanguage } from '@codemirror/language'
import { nginx } from '@codemirror/legacy-modes/mode/nginx'

const nginxLanguage = () => StreamLanguage.define(nginx)
export default nginxLanguage