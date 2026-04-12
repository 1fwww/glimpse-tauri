import { useState, useCallback, useEffect, useRef } from 'react'

const STALE_THRESHOLD = 5 * 60 * 1000

function newThread() {
  return { id: null, title: 'New Chat', messages: [], createdAt: Date.now(), updatedAt: Date.now() }
}

export default function useThreadManager() {
  const [currentThread, setCurrentThread] = useState(null)
  const currentThreadRef = useRef(null)
  const [recentThreads, setRecentThreads] = useState([])
  const [isNewThread, setIsNewThread] = useState(false)
  const [provider, setProviderState] = useState(() => localStorage.getItem('glimpse-provider') || 'claude')
  const [modelId, setModelIdState] = useState(() => localStorage.getItem('glimpse-model') || null)
  const [availableProviders, setAvailableProviders] = useState([])

  // Keep ref in sync for closures that can't depend on state
  useEffect(() => { currentThreadRef.current = currentThread }, [currentThread])

  const setProvider = useCallback((p) => { localStorage.setItem('glimpse-provider', p); setProviderState(p) }, [])
  const setModelId = useCallback((m) => { localStorage.setItem('glimpse-model', m); setModelIdState(m) }, [])

  // Load threads + providers on mount
  useEffect(() => {
    const load = () => {
      if (!window.electronAPI?.getThreads) {
        setTimeout(load, 100)
        return
      }
      window.electronAPI.getAvailableProviders?.().then(providers => {
        setAvailableProviders(providers || [])
        if (providers?.length > 0) {
          const allModels = providers.flatMap(p => p.models?.map(m => ({ ...m, providerId: p.id })) || [])
          const savedModel = localStorage.getItem('glimpse-model')
          const savedExists = allModels.find(m => m.id === savedModel)
          if (!savedExists) {
            setProvider(providers[0].id)
            setModelId(providers[0].models?.[0]?.id || null)
          }
        }
      })
      window.electronAPI.getThreads().then(threads => {
        setRecentThreads(threads || [])
        const mostRecent = threads?.[0]
        if (mostRecent && (Date.now() - mostRecent.updatedAt) < STALE_THRESHOLD) {
          setCurrentThread(mostRecent)
          setIsNewThread(false)
        } else {
          setCurrentThread(newThread())
          setIsNewThread(true)
        }
      })
    }
    load()
  }, [])


  const handleThreadChange = useCallback((thread) => {
    setCurrentThread(thread)
    setIsNewThread(false)
    // Existing thread with messages → expanded, empty → compact
    if (thread?.messages?.length > 0) {
      window.electronAPI?.resizeChatWindow?.({ width: 380, height: 550, animate: false })
    } else {
      window.electronAPI?.resizeChatWindow?.({ width: 380, height: 412, animate: false })
    }
  }, [])

  const handleNewThread = useCallback(() => {
    setCurrentThread(newThread())
    setIsNewThread(true)
    // Keep current window size — don't collapse on new chat
  }, [])

  const handleSetCurrentThread = useCallback((thread) => {
    setCurrentThread(thread)
    window.electronAPI?.getThreads?.().then(threads => {
      setRecentThreads(threads || [])
    })
  }, [])

  const refreshThreads = useCallback(() => {
    window.electronAPI?.getThreads?.().then(threads => {
      setRecentThreads(threads || [])
    })
  }, [])

  const handleClearAllThreads = useCallback(async () => {
    for (const t of recentThreads) {
      await window.electronAPI?.deleteThread(t.id)
    }
    setRecentThreads([])
    setCurrentThread(newThread())
    setIsNewThread(true)
    window.electronAPI?.resizeChatWindow?.({ width: 380, height: 412, animate: false })
    window.electronAPI?.refreshTrayMenu?.()
  }, [recentThreads])

  // Refresh threads with time heuristic + resize (called on chat show)
  const refreshOnShow = useCallback(() => {
    // If current thread has no messages (empty new chat), stay compact — don't restore
    const cur = currentThreadRef.current
    if (cur && (!cur.messages || cur.messages.length === 0)) {
      window.electronAPI?.resizeChatWindow?.({ width: 380, height: 412, animate: false })
      return
    }

    window.electronAPI?.getThreads?.().then(threads => {
      setRecentThreads(threads || [])
      const mostRecent = threads?.[0]
      if (mostRecent && (Date.now() - mostRecent.updatedAt) < STALE_THRESHOLD) {
        // Back to existing thread — expanded
        setCurrentThread(mostRecent)
        setIsNewThread(false)
        window.electronAPI?.resizeChatWindow?.({ width: 380, height: 550, animate: false })
      } else {
        // New thread — compact
        setCurrentThread(newThread())
        setIsNewThread(true)
        window.electronAPI?.resizeChatWindow?.({ width: 380, height: 412, animate: false })
      }
    })
  }, [])

  // Refresh threads with time heuristic (called by overlay on screen capture)
  const refreshWithHeuristic = useCallback(() => {
    window.electronAPI?.getThreads?.().then(threads => {
      setRecentThreads(threads || [])
      const mostRecent = threads?.[0]
      if (mostRecent && (Date.now() - mostRecent.updatedAt) < STALE_THRESHOLD) {
        setCurrentThread(mostRecent)
        setIsNewThread(false)
      } else {
        setCurrentThread(newThread())
        setIsNewThread(true)
      }
    })
  }, [])

  const refreshProviders = useCallback(async () => {
    const providers = await window.electronAPI?.getAvailableProviders()
    setAvailableProviders(providers || [])
    if (providers?.length > 0) {
      const allModels = providers.flatMap(p => p.models?.map(m => ({ ...m, providerId: p.id })) || [])
      const savedModel = localStorage.getItem('glimpse-model')
      const savedExists = allModels.find(m => m.id === savedModel)
      if (!savedExists) {
        setProvider(providers[0].id)
        setModelId(providers[0].models?.[0]?.id || null)
      }
    }
  }, [])

  return {
    currentThread,
    setCurrentThread: handleSetCurrentThread,
    recentThreads,
    isNewThread,
    setIsNewThread,
    provider,
    setProvider,
    modelId,
    setModelId,
    availableProviders,
    handleThreadChange,
    handleNewThread,
    handleClearAllThreads,
    refreshThreads,
    refreshWithHeuristic,
    refreshOnShow,
    refreshProviders,
  }
}
