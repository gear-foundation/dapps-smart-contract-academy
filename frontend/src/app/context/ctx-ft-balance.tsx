import type { ProviderProps } from '@gear-js/react-hooks'
import { createContext } from 'react'
import { ProgramMetadata } from '@gear-js/api'
import meta from '@/assets/meta/ft_main.meta.txt'
import metaLogic from '@/assets/meta/ft_logic.meta.txt'
import metaStorage from '@/assets/meta/ft_storage.meta.txt'
import { HexString } from '@polkadot/util/types'
import { ENV } from '@/app/consts'
import { useMetadata } from '@/app/hooks/use-metadata'

type Program = {
  programId: HexString
  metaMain: ProgramMetadata
  metaLogic: ProgramMetadata
  metaStorage: ProgramMetadata
}

const program: Program = {
  programId: '' as HexString,
  metaMain: {} as ProgramMetadata,
  metaLogic: {} as ProgramMetadata,
  metaStorage: {} as ProgramMetadata,
}

function useStore(): Program {
  const { metadata } = useMetadata(meta)
  const { metadata: metaL } = useMetadata(metaLogic)
  const { metadata: metaS } = useMetadata(metaStorage)
  return {
    programId: ENV.balance as HexString,
    metaMain: metadata as ProgramMetadata,
    metaLogic: metaL as ProgramMetadata,
    metaStorage: metaS as ProgramMetadata,
  }
}

export const FTBalanceCtx = createContext<Program>(program)

export function TokensBalanceProvider({ children }: ProviderProps) {
  const { Provider } = FTBalanceCtx
  return <Provider value={useStore()}>{children}</Provider>
}
