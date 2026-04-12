import React, { useState, useEffect } from 'react'
import ChatPanel from './ChatPanel'
import useThreadManager from './useThreadManager'
import './app.css'

export default function ChatOnlyApp() {
  const [initialContext, setInitialContext] = useState({ text: '', seq: 0 })
  const [isPinned, setIsPinned] = useState(false)
  const [croppedImage, setCroppedImage] = useState(null)
  const tm = useThreadManager()

  useEffect(() => {
    window.electronAPI?.onTextContext?.((text) => setInitialContext({ text: text?.trim() || '', seq: Date.now() }))
    window.electronAPI?.onClearTextContext?.(() => setInitialContext(prev => ({ text: '', seq: prev.seq + 1 })))
    window.electronAPI?.onPinState?.((state) => setIsPinned(state))
    window.electronAPI?.onSetCroppedImage?.((img) => setCroppedImage(img))
    window.electronAPI?.onClearScreenshot?.(() => setCroppedImage(null))
    // Chat shown — re-evaluate thread (5min heuristic) and resize accordingly
    window.electronAPI?.onCheckSize?.(() => tm.refreshOnShow())
    // Stale chat — Swift determined >5min elapsed, start fresh (lightweight, no re-render)
    window.electronAPI?.onStartNewThread?.(() => tm.handleNewThread())
    // Receive full thread data from pin (no disk read needed)
    window.electronAPI?.onLoadThreadData?.((data) => {
      if (data) {
        // Pin sends { thread, croppedImage }, open-thread sends thread directly
        // Use setCurrentThread (not handleThreadChange) to avoid resizeChatWindow —
        // in chat-only mode the panel fills the window, Swift controls the size.
        const thread = data.thread || data
        tm.setCurrentThread(thread)
        if (thread?.messages?.length > 0) tm.setIsNewThread(false)
        if (data.croppedImage) setCroppedImage(data.croppedImage)
      }
    })
  }, [])

  const handleClose = () => {
    window.electronAPI?.closeChatWindow?.()
  }

  // ESC to close — capture phase
  useEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        e.stopPropagation()
        handleClose()
      }
    }
    window.addEventListener('keydown', handleKeyDown, true)
    return () => window.removeEventListener('keydown', handleKeyDown, true)
  }, [])

  // Signal main process that React has rendered
  useEffect(() => {
    window.electronAPI?.chatReady?.()
  }, [])

  if (!tm.currentThread) return null

  return (
    <div className="chat-only-app">
      <div className={`chat-only-inner ${isPinned ? 'pinned' : ''}`}>
      <ChatPanel
        style={{}}
        croppedImage={croppedImage}
        getCompositeImage={null}
        onDismissScreenshot={() => setCroppedImage(null)}
        currentThread={tm.currentThread}
        setCurrentThread={tm.setCurrentThread}
        recentThreads={tm.recentThreads}
        onThreadChange={tm.handleThreadChange}
        onNewThread={tm.handleNewThread}
        onClearAllThreads={tm.handleClearAllThreads}
        chatFullSize={true}
        setChatFullSize={() => {}}
        isNewThread={tm.isNewThread}
        setIsNewThread={tm.setIsNewThread}
        refreshThreads={tm.refreshThreads}
        refreshProviders={tm.refreshProviders}
        onClose={handleClose}
        onMinimize={handleClose}
        provider={tm.provider}
        setProvider={tm.setProvider}
        modelId={tm.modelId}
        setModelId={tm.setModelId}
        availableProviders={tm.availableProviders}
        annotationCount={0}
        initialContext={initialContext}
        isPinned={isPinned}
        onTogglePin={() => window.electronAPI?.togglePin?.()}
      />
      </div>
    </div>
  )
}
