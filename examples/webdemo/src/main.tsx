import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { AudioGraphProvider } from './audio-graph/provider.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AudioGraphProvider>
      <App />
    </AudioGraphProvider>
  </StrictMode>,
)
