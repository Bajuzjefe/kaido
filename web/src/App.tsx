import { Routes, Route } from 'react-router-dom'
import { lazy, Suspense } from 'react'
import Layout from './components/Layout'
import Landing from './pages/Landing'

// Lazy-load heavy pages for faster initial load
const Wizard = lazy(() => import('./pages/Wizard'))
const Docs = lazy(() => import('./pages/Docs'))

function PageFallback() {
  return (
    <div className="flex items-center justify-center min-h-[60vh]">
      <div className="animate-pulse text-on-surface-variant text-sm">Loading...</div>
    </div>
  )
}

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Landing />} />
        <Route path="wizard" element={<Suspense fallback={<PageFallback />}><Wizard /></Suspense>} />
        <Route path="docs/*" element={<Suspense fallback={<PageFallback />}><Docs /></Suspense>} />
      </Route>
    </Routes>
  )
}
