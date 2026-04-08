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
    window.electronAPI?.onTextContext?.((text) => setInitialContext(prev => ({ text, seq: prev.seq + 1 })))
    window.electronAPI?.onPinState?.((state) => setIsPinned(state))
    window.electronAPI?.onSetCroppedImage?.((img) => setCroppedImage(img))
    // Receive full thread data from pin (no disk read needed)
    window.electronAPI?.onLoadThreadData?.((data) => {
      if (data) {
        // Pin sends { thread, croppedImage }, open-thread sends thread directly
        const thread = data.thread || data
        tm.handleThreadChange(thread)
        if (data.croppedImage) setCroppedImage(data.croppedImage)
      }
    })
  }, [])

  const handleClose = () => {
    window.electronAPI?.closeChatWindow?.()
    window.close()
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
    if (tm.currentThread) window.electronAPI?.chatReady?.()
  }, [!!tm.currentThread])

  if (!tm.currentThread) return null

  return (
    <div className={`chat-only-app ${isPinned ? 'pinned' : ''}`}>
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
  )
}
