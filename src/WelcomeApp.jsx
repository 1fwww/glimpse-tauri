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

  const [prevStep, setPrevStep] = useState(step)
  const stepDirection = step > prevStep ? 'forward' : 'back'
  useEffect(() => { setPrevStep(step) }, [step])

  return (
    <div className="welcome-app">
      <h1 className="sr-only">Glimpse Setup</h1>
      <div className="welcome-drag-bar">
        <button className="welcome-close" onClick={handleClose} aria-label="Close">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div className="welcome-content">
        {step === 0 && (
          <div className={`welcome-step step-${stepDirection}`}>
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
            <p className="welcome-title">Glimpse</p>
            <p className="welcome-desc">Glimpse your screen. Stay in flow.</p>
            <button className="welcome-btn" onClick={() => setStep(1)}>Get Started</button>
          </div>
        )}

        {step === 1 && (
          <div className={`welcome-step step-${stepDirection}`}>
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
                  <svg className="perm-check check-draw check-icon" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
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
                  <svg className="perm-check check-draw check-icon" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
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
          <div className={`welcome-step step-${stepDirection}`}>
            <h2 className="welcome-subtitle">Two ways to start</h2>
            <p className="welcome-desc">Try each shortcut now.</p>

            <div className="welcome-shortcuts">
              <div className={`shortcut-row ${triedShortcuts.screenshot ? 'shortcut-done' : !triedShortcuts.screenshot ? 'shortcut-pulse' : ''}`}>
                <div className="shortcut-status">
                  {triedShortcuts.screenshot ? (
                    <svg className="check-draw check-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
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
                    <svg className="check-draw check-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.5"><path d="M20 6L9 17l-5-5"/></svg>
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
          <div className={`welcome-step welcome-step-pin step-${stepDirection}`}>
            <h2 className="welcome-subtitle">Pin to screen</h2>
            <p className="welcome-desc">Keep the chat visible while you work.</p>

            <div className="pin-demo">
              <div className="pin-demo-bg">
                {[95, 80, 95, 60, 0, 95, 80, 60].map((w, i) => (
                  <div key={i} className="pin-demo-line" style={w ? { width: `${w}%` } : { height: 12, background: 'transparent' }} />
                ))}
              </div>
              <div className={`pin-demo-panel ${pinnedEgg ? 'pin-demo-pinned' : ''}`}>
                <div className="pin-demo-header">
                  <svg viewBox="60 140 420 280" width={16} height={11} style={{ flexShrink: 0 }}>
                    <path d="M98 212C152 174 365 158 420 248" fill="none" stroke="#6C63FF" strokeWidth="20" strokeLinecap="round" style={{ opacity: pinnedEgg ? 0 : 1, transition: 'opacity 0.3s ease' }} />
                    <path d="M98 192C200 192 350 204 420 234" fill="none" stroke="#6C63FF" strokeWidth="20" strokeLinecap="round" style={{ opacity: pinnedEgg ? 1 : 0, transition: 'opacity 0.3s ease' }} />
                    <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="#6C63FF" strokeWidth="22" strokeLinecap="round" />
                  </svg>
                  <span className="pin-demo-title">New Chat</span>
                  <div style={{ flex: 1 }} />
                  <button className={`pin-demo-pin ${pinnedEgg ? 'pin-demo-pin-active' : ''}`} onClick={(e) => { e.stopPropagation(); setPinnedEgg(p => !p) }}>
                    <svg viewBox="0 0 24 24" width={11} height={11} fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" style={{ transform: pinnedEgg ? 'rotate(0deg)' : 'rotate(20deg)', transition: 'transform 0.3s cubic-bezier(0.22, 1, 0.36, 1)' }}>
                      <path d="M12 17v5" /><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.89A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.89A2 2 0 0 0 5 15.24z" />
                    </svg>
                  </button>
                </div>
                <div className="pin-demo-messages">
                  <div className="pin-demo-bubble pin-demo-user">What does this error mean?</div>
                  <div className="pin-demo-bubble pin-demo-asst">That's a type mismatch — the function expects a string.</div>
                </div>
                <div className="pin-demo-input">
                  <div className="pin-demo-input-field">Ask anything...</div>
                </div>
              </div>
            </div>
            <p className="pin-demo-hint">{pinnedEgg ? "You got it. That's all you need." : '\u00A0'}</p>

            <p className="welcome-hint">Add an API key in <button className="welcome-hint-link" onClick={() => window.electronAPI?.openSettings?.()}>Settings</button> or use an invite code to chat.</p>

            <button className="welcome-btn" onClick={handleGetStarted}>Start Using Glimpse</button>
          </div>
        )}

      </div>
      <div className="welcome-footer">
        <div className="welcome-dots" role="tablist" aria-label="Setup steps">
          {['Intro', 'Permissions', 'Shortcuts', 'Pin'].map((label, i) => (
            <button
              key={i}
              role="tab"
              aria-label={`Step ${i + 1}: ${label}`}
              aria-selected={step === i}
              className={`welcome-dot ${step === i ? 'active' : ''} ${step === 3 && i === 3 ? 'dot-pulse' : ''}`}
              onClick={() => { if (i <= step) setStep(i) }}
              disabled={i > step}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
