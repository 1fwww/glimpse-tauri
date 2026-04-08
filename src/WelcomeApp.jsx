import React, { useState, useEffect } from 'react'
import './app.css'

export default function WelcomeApp() {
  const [step, setStep] = useState(() => {
    const saved = localStorage.getItem('welcome-step')
    return saved ? parseInt(saved, 10) : 0
  })
  const [permissions, setPermissions] = useState({ screen: false, accessibility: false })
  const [checking, setChecking] = useState(false)
  const [triedShortcuts, setTriedShortcuts] = useState({ screenshot: false, chat: false })
  const [showShortcutSkip, setShowShortcutSkip] = useState(false)
  const [pinnedEgg, setPinnedEgg] = useState(false)
  const [showPermSkip, setShowPermSkip] = useState(false)

  // Show skip links after 5 seconds on permission/shortcut steps
  useEffect(() => {
    if (step === 1) {
      setShowPermSkip(false)
      const timer = setTimeout(() => setShowPermSkip(true), 5000)
      return () => clearTimeout(timer)
    }
  }, [step])

  useEffect(() => {
    if (step === 2) {
      setShowShortcutSkip(false)
      const timer = setTimeout(() => setShowShortcutSkip(true), 5000)
      return () => clearTimeout(timer)
    }
  }, [step])

  // Persist step across restarts
  useEffect(() => {
    localStorage.setItem('welcome-step', step.toString())
  }, [step])

  const checkPermissions = async () => {
    setChecking(true)
    const result = await window.electronAPI?.checkPermissions()
    if (result) setPermissions(result)
    setChecking(false)
    return result
  }

  useEffect(() => {
    // On mount, check permissions and force back to step 1 if not granted
    checkPermissions().then(result => {
      if (result && (!result.screen || !result.accessibility)) {
        setStep(prev => prev > 1 ? 1 : prev)
      }
    })
    const interval = setInterval(checkPermissions, 2000)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    window.electronAPI?.onShortcutTried?.((type) => {
      setTriedShortcuts(prev => ({ ...prev, [type]: true }))
    })
  }, [])

  const allGranted = permissions.screen && permissions.accessibility

  const handleGetStarted = () => {
    localStorage.removeItem('welcome-step')
    window.electronAPI?.welcomeDone()
  }

  const handleClose = () => {
    window.electronAPI?.closeWelcome?.()
  }

  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key === 'Escape') handleClose()
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  return (
    <div className="welcome-app">
      <div className="welcome-drag-bar">
        <button className="welcome-close" onClick={handleClose} aria-label="Close">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div className="welcome-content">
        {step === 0 && (
          <div className="welcome-step">
            <div
              className="welcome-logo glimpse-icon-fixed logo-draw-blink"
              onClick={(e) => {
                const el = e.currentTarget
                el.classList.remove('logo-draw-blink')
                void el.offsetWidth
                el.classList.add('logo-draw-blink')
              }}
              style={{ cursor: 'pointer' }}
            >
              <svg viewBox="60 140 420 280" width={64} height={43}>
                <path d="M98 212C152 174 365 158 420 248" fill="none" stroke="#6C63FF" strokeWidth="20" strokeLinecap="round" />
                <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="#6C63FF" strokeWidth="22" strokeLinecap="round" />
              </svg>
            </div>
            <h1 className="welcome-title">Glimpse</h1>
            <p className="welcome-desc">Glimpse your screen. Stay in flow.</p>
            <button className="welcome-btn" onClick={() => setStep(1)}>Get Started</button>
          </div>
        )}

        {step === 1 && (
          <div className="welcome-step">
            <h2 className="welcome-subtitle">Quick setup</h2>
            <p className="welcome-desc">Glimpse needs two permissions to capture your screen and respond to shortcuts.</p>

            <div className="welcome-permissions">
              <div className={`welcome-perm ${permissions.screen ? 'granted' : ''}`}>
                <div className="perm-icon">
                  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>
                </div>
                <div className="perm-info">
                  <div className="perm-name">Screen Recording</div>
                  <div className="perm-why">Required to capture screenshots</div>
                </div>
                {permissions.screen ? (
                  <svg className="perm-check check-draw" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#238542" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
                ) : (
                  <button className="perm-grant" onClick={async () => {
                    const result = await window.electronAPI?.requestScreenPermission()
                    if (result?.granted) checkPermissions()
                  }}>Grant</button>
                )}
              </div>

              <div className={`welcome-perm ${permissions.accessibility ? 'granted' : ''}`}>
                <div className="perm-icon">
                  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
                </div>
                <div className="perm-info">
                  <div className="perm-name">Accessibility</div>
                  <div className="perm-why">Required for global keyboard shortcuts</div>
                </div>
                {permissions.accessibility ? (
                  <svg className="perm-check check-draw" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#238542" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
                ) : (
                  <button className="perm-grant" onClick={() => window.electronAPI?.openPermissionSettings('accessibility')}>Grant</button>
                )}
              </div>
            </div>

            <button className="welcome-btn" onClick={() => setStep(2)} disabled={!allGranted}>
              {allGranted ? 'Continue' : 'Grant both to continue'}
            </button>
            <div style={{ minHeight: 28 }}>
              {!allGranted && showPermSkip && (
                <button className="welcome-skip-link" onClick={() => setStep(2)}>
                  I've granted permissions, continue
                </button>
              )}
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="welcome-step">
            <h2 className="welcome-subtitle">Two ways to start</h2>
            <p className="welcome-desc">Try each shortcut now.</p>

            <div className="welcome-shortcuts">
              <div className={`shortcut-row ${triedShortcuts.screenshot ? 'shortcut-done' : !triedShortcuts.screenshot ? 'shortcut-pulse' : ''}`}>
                <div className="shortcut-status">
                  {triedShortcuts.screenshot ? (
                    <svg className="check-draw" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#238542" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
                  ) : (
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="12" cy="12" r="8"/></svg>
                  )}
                </div>
                <div className="shortcut-info">
                  <div className="shortcut-name">Start with a screenshot</div>
                  <div className="shortcut-desc">Capture, annotate, ask AI</div>
                </div>
                <div className="shortcut-keys">
                  <kbd>Cmd</kbd><kbd>Shift</kbd><kbd>Z</kbd>
                </div>
              </div>
              <div className={`shortcut-row ${triedShortcuts.chat ? 'shortcut-done' : triedShortcuts.screenshot ? 'shortcut-pulse' : ''}`}>
                <div className="shortcut-status">
                  {triedShortcuts.chat ? (
                    <svg className="check-draw" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#238542" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
                  ) : (
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="12" cy="12" r="8"/></svg>
                  )}
                </div>
                <div className="shortcut-info">
                  <div className="shortcut-name">Start with text</div>
                  <div className="shortcut-desc">Select text, ask AI about it</div>
                </div>
                <div className="shortcut-keys">
                  <kbd>Cmd</kbd><kbd>Shift</kbd><kbd>X</kbd>
                </div>
              </div>
            </div>

            <button className={`welcome-btn ${triedShortcuts.screenshot && triedShortcuts.chat ? 'btn-pop' : ''}`} onClick={() => setStep(3)} disabled={!triedShortcuts.screenshot || !triedShortcuts.chat}>
              {triedShortcuts.screenshot && triedShortcuts.chat ? 'Continue' : 'Try both to continue'}
            </button>
            <div style={{ minHeight: 28 }}>
              {showShortcutSkip && !(triedShortcuts.screenshot && triedShortcuts.chat) && (
                <button className="welcome-skip-link" onClick={() => setStep(3)}>
                  Shortcuts not working? Skip
                </button>
              )}
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="welcome-step">
            <h2 className="welcome-subtitle">Pin to screen</h2>
            <p className="welcome-desc">Keep the chat visible while you work.</p>

            <div className="welcome-features">
              <div
                className={`welcome-feature-row ${pinnedEgg ? 'welcome-feature-pinned' : ''}`}
                style={{ cursor: 'pointer' }}
                onClick={() => setPinnedEgg(p => !p)}
              >
                <div className="welcome-feature-icon">
                  <svg viewBox="0 0 24 24" width={20} height={20} fill="none" stroke={pinnedEgg ? '#6C63FF' : '#6C63FF'} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M12 17v5" /><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24z" />
                  </svg>
                </div>
                <div className="welcome-feature-info">
                  <div className="welcome-feature-name">{pinnedEgg ? 'Pinned!' : 'Pin your chat'}</div>
                  <div className="welcome-feature-desc">{pinnedEgg ? "You got it. That's all you need." : 'Try it — click this card.'}</div>
                </div>
              </div>
            </div>

            <p className="welcome-hint">To chat with AI, add an API key in <strong>Settings</strong>. You can also use an invite code.</p>

            <button className="welcome-btn" onClick={handleGetStarted}>Start Using Glimpse</button>
          </div>
        )}

      </div>
      <div className="welcome-footer">
        <div className="welcome-dots">
          {[0, 1, 2, 3].map(i => (
            <div key={i} className={`welcome-dot ${step === i ? 'active' : ''}`} onClick={() => { if (i <= step) setStep(i) }} />
          ))}
        </div>
      </div>
    </div>
  )
}
