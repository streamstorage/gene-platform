import { type User } from './schema'
import { type InjectionKey } from 'vue'

export interface UserActions {
  openUserEditDialog: (user: User | undefined) => void
  openUserActivateDialog: (activate: boolean, user: Array<User>) => void
}

export const userActionsKey: InjectionKey<UserActions> = Symbol('userActions')
