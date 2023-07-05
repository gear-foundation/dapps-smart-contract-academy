import { createContext, ReactNode, useEffect, useRef, useState } from 'react'
import { ProgramMetadata } from '@gear-js/api'
import { LessonState } from '@/app/types/lessons'
import { useLessonAssets } from '@/app/utils/get-lesson-assets'

const key = 'tmgState'

const useProgram = () => {
  const [lesson, setLesson] = useState<LessonState>()
  const [lessonMeta, setLessonMeta] = useState<ProgramMetadata>()
  const [isAdmin, setIsAdmin] = useState<boolean>(false)
  const [isReady, setIsReady] = useState<boolean>(false)
  const isParsed = useRef(false)
  const resetLesson = () => {
    setLesson(undefined)
    setIsAdmin(false)
    setIsReady(false)
    localStorage.removeItem(key)
  }

  const assets = useLessonAssets()

  useEffect(() => {
    if (lesson && assets.every(Boolean)) {
      localStorage.setItem(key, JSON.stringify(lesson))
      setLessonMeta(assets[+lesson.step || 0])
    } else {
      if (!isParsed.current) {
        const ls = localStorage.getItem(key)
        if (ls) {
          setLesson(JSON.parse(ls))
          isParsed.current = true
        }
      } else localStorage.removeItem(key)
    }
  }, [assets, lesson])

  return {
    lesson,
    setLesson,
    lessonMeta,
    isAdmin,
    setIsAdmin,
    isReady,
    setIsReady,
    resetLesson,
  }
}

export const LessonsCtx = createContext({} as ReturnType<typeof useProgram>)

export function LessonsProvider({ children }: { children: ReactNode }) {
  const { Provider } = LessonsCtx
  return <Provider value={useProgram()}>{children}</Provider>
}
