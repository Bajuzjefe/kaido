import { Link, Outlet, useLocation } from 'react-router-dom'
import { useState, useMemo, useEffect } from 'react'

const NAV_LINKS = [
  { to: '/wizard', label: 'Wizard' },
  { to: '/docs', label: 'Docs' },
]

export default function Layout() {
  const { pathname } = useLocation()
  const [mobileOpen, setMobileOpen] = useState(false)
  const isFullWidth = useMemo(() => pathname.startsWith('/wizard') || pathname.startsWith('/docs'), [pathname])
  const isWizard = useMemo(() => pathname.startsWith('/wizard'), [pathname])

  // Scroll to top on route change
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [pathname])

  return (
    <div className="min-h-screen flex flex-col">
      <header className="bg-surface border-b border-outline-variant">
        <div className={`${isFullWidth ? 'px-4' : 'max-w-7xl mx-auto px-6 lg:px-12'} h-14 flex items-center gap-8`}>
          {/* Logo */}
          <Link to="/" className="flex items-center gap-2 shrink-0">
            <svg width="28" height="28" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M16 2L28 9.5V22.5L16 30L4 22.5V9.5L16 2Z" fill="#E8EAF0"/>
              <path d="M16 8L22 11.5V18.5L16 22L10 18.5V11.5L16 8Z" fill="#0F1012"/>
            </svg>
            <span className="text-lg font-semibold text-on-surface">kaido</span>
          </Link>

          {/* Desktop Nav */}
          <nav className="hidden md:flex items-center gap-1 flex-1">
            {NAV_LINKS.map(({ to, label }) => {
              const active = to === '/' ? pathname === '/' : pathname.startsWith(to)
              return (
                <Link
                  key={to}
                  to={to}
                  className={`px-4 py-1.5 rounded-full text-sm transition-colors whitespace-nowrap ${
                    active
                      ? 'text-on-surface font-medium bg-surface-container-high'
                      : 'text-on-surface-variant hover:text-on-surface hover:bg-nav-hover'
                  }`}
                >
                  {label}
                </Link>
              )
            })}
          </nav>

          {/* CTA + GitHub */}
          <div className="hidden md:flex items-center gap-3 ml-auto">
            <a
              href="https://github.com/Bajuzjefe/kaido"
              target="_blank"
              rel="noopener noreferrer"
              className="px-4 py-1.5 rounded-full text-sm text-on-surface-variant hover:text-on-surface hover:bg-nav-hover transition-colors"
            >
              GitHub
            </a>
            <Link
              to="/wizard"
              className="btn-primary text-sm"
            >
              Start Building
            </Link>
          </div>

          {/* Mobile toggle */}
          <button
            onClick={() => setMobileOpen(!mobileOpen)}
            className="md:hidden ml-auto w-10 h-10 rounded-full flex items-center justify-center hover:bg-nav-hover transition-colors"
            aria-label="Toggle menu"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              {mobileOpen ? (
                <path d="M6 6l12 12M6 18L18 6" />
              ) : (
                <path d="M3 12h18M3 6h18M3 18h18" />
              )}
            </svg>
          </button>
        </div>

        {/* Mobile nav */}
        {mobileOpen && (
          <nav className="md:hidden border-t border-outline-variant bg-surface px-6 py-4 space-y-1">
            {NAV_LINKS.map(({ to, label }) => (
              <Link
                key={to}
                to={to}
                onClick={() => setMobileOpen(false)}
                className="block px-4 py-3 text-lg font-light text-on-surface-variant hover:text-on-surface border-b border-outline-variant"
              >
                {label}
              </Link>
            ))}
            <a
              href="https://github.com/Bajuzjefe/kaido"
              target="_blank"
              rel="noopener noreferrer"
              className="block px-4 py-3 text-lg font-light text-on-surface-variant"
            >
              GitHub
            </a>
          </nav>
        )}
      </header>

      <main className="flex-1">
        <Outlet />
      </main>

      {/* Minimal footer (hidden on wizard) */}
      {!isWizard && <footer className="border-t border-outline-variant py-8 mt-auto">
        <div className="max-w-7xl mx-auto px-6 lg:px-12 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-on-surface-variant">
          <div className="flex items-center gap-2">
            <svg width="18" height="18" viewBox="0 0 32 32" fill="none">
              <path d="M16 2L28 9.5V22.5L16 30L4 22.5V9.5L16 2Z" fill="#E8EAF0"/>
              <path d="M16 8L22 11.5V18.5L16 22L10 18.5V11.5L16 8Z" fill="#0F1012"/>
            </svg>
            <span>kaido - Secure smart contracts for Cardano</span>
          </div>
          <div className="flex items-center gap-6">
            <Link to="/docs" className="hover:text-on-surface transition-colors">Docs</Link>
            <a href="https://github.com/Bajuzjefe/kaido" target="_blank" rel="noopener noreferrer" className="hover:text-on-surface transition-colors">GitHub</a>
          </div>
        </div>
      </footer>}
    </div>
  )
}
