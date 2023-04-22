import {
  createContext,
  Dispatch,
  ReactNode,
  SetStateAction,
  useEffect,
  useRef,
  useState,
} from 'react'
import { getProgramMetadata, ProgramMetadata } from '@gear-js/api'
import { LessonState } from '@/app/types/lessons'
import { getLessonAssets } from '@/app/utils/get-lesson-assets'

type Program = {
  lesson?: LessonState
  setLesson: Dispatch<SetStateAction<LessonState | undefined>>
  lessonMeta?: ProgramMetadata
  isAdmin: boolean
  setIsAdmin: Dispatch<SetStateAction<boolean>>
  isReady: boolean
  setIsReady: Dispatch<SetStateAction<boolean>>
  resetLesson: () => void
}

export const LessonsCtx = createContext({} as Program)

const key = 'tmgState'

const useLesson = (): Program => {
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

  useEffect(() => {
    if (lesson) {
      localStorage.setItem(key, JSON.stringify(lesson))

      fetch(getLessonAssets(+lesson.step))
        .then((res) => res.text())
        .then((raw) => getProgramMetadata(`0x${raw}`))
        .then((meta) => setLessonMeta(meta))
        .catch((e) => console.log('error', e))
    } else {
      if (!isParsed.current) {
        const ls = localStorage.getItem(key)
        if (ls) {
          setLesson(JSON.parse(ls))
          isParsed.current = true
        }
      } else localStorage.removeItem(key)
    }
  }, [lesson])

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

export function LessonsProvider({ children }: { children: ReactNode }) {
  const { Provider } = LessonsCtx
  return <Provider value={useLesson()}>{children}</Provider>
}
