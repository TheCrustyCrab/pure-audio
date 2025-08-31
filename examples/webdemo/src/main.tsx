import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { AudioGraphProvider } from './audio-graph/provider.tsx'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <AudioGraphProvider>
      <App/>
    </AudioGraphProvider>
  </StrictMode>,
)
