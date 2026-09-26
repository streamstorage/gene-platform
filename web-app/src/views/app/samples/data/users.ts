import { type User } from './schema'
import { faker } from '@faker-js/faker'

// Set a fixed seed for consistent data generation
faker.seed(67890)

export const users = Array.from({ length: 500 }, () => {
  const firstName = faker.person.firstName()
  const lastName = faker.person.lastName()
  return {
    id: faker.string.uuid(),
    name: `${firstName} ${lastName}`,
    email: faker.internet.email({ firstName }).toLocaleLowerCase(),
    notes: faker.person.jobDescriptor(),
    status: faker.helpers.arrayElement(['active', 'inactive']),
    role: faker.helpers.arrayElement(['admin', 'user']),
    createdAt: faker.date.past(),
    updatedAt: faker.date.recent(),
  }
}) satisfies User[]
