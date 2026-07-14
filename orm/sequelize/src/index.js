import { readFileSync, statSync, writeFileSync, mkdirSync } from 'fs'
import { join } from 'path'

export function createORMAdapter() {
  return {
    name: 'sequelize',
    detect(dir) {
      try {
        const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'))
        return pkg?.dependencies?.sequelize || pkg?.devDependencies?.sequelize
      } catch { return false }
    },
    supportedVersions: ['6.37'],
    defaultVersion: '6.37',
    kind: 'ORM',

    scaffold(name, options) {
      return scaffoldSequelize(name, options)
    }
  }
}

function existsSync(p) {
  try { return statSync(p).isFile() } catch { return false }
}

async function scaffoldSequelize(name, options) {
  const projectDir = join(options.dir || process.cwd(), name)
  mkdirSync(join(projectDir, 'src', 'models'), { recursive: true })

  writeFileSync(join(projectDir, 'package.json'), JSON.stringify({
    name, version: '1.0.0', private: true, type: 'module',
    scripts: { dev: 'node src/index.js', 'db:migrate': 'sequelize db:migrate', 'db:seed': 'sequelize db:seed:all' },
    dependencies: { sequelize: '^6.37.0', pg: '^8.13.0' }
  }, null, 2))

  writeFileSync(join(projectDir, 'src', 'models', 'index.js'), `import { Sequelize } from 'sequelize'
import UserModel from './User.js'
import PostModel from './Post.js'

const sequelize = new Sequelize(process.env.DATABASE_URL || 'postgresql://user:password@localhost:5432/${name}', {
  dialect: 'postgres',
  logging: false,
})

const db = {
  sequelize,
  Sequelize,
  User: UserModel(sequelize),
  Post: PostModel(sequelize),
}

// Define associations
db.User.hasMany(db.Post, { foreignKey: 'authorId' })
db.Post.belongsTo(db.User, { foreignKey: 'authorId' })

export default db
`)

  writeFileSync(join(projectDir, 'src', 'models', 'User.js'), `import { DataTypes } from 'sequelize'

export default function (sequelize) {
  return sequelize.define('User', {
    id: { type: DataTypes.INTEGER, primaryKey: true, autoIncrement: true },
    email: { type: DataTypes.STRING, allowNull: false, unique: true },
    name: { type: DataTypes.STRING },
  }, {
    timestamps: true,
  })
}
`)

  writeFileSync(join(projectDir, 'src', 'models', 'Post.js'), `import { DataTypes } from 'sequelize'

export default function (sequelize) {
  return sequelize.define('Post', {
    id: { type: DataTypes.INTEGER, primaryKey: true, autoIncrement: true },
    title: { type: DataTypes.STRING, allowNull: false },
    content: { type: DataTypes.TEXT },
    published: { type: DataTypes.BOOLEAN, defaultValue: false },
  }, {
    timestamps: true,
  })
}
`)

  writeFileSync(join(projectDir, 'src', 'index.js'), `import db from './models/index.js'

await db.sequelize.authenticate()
console.log('Database connected')

const users = await db.User.findAll({ include: db.Post })
console.log(JSON.stringify(users, null, 2))

await db.sequelize.close()
`)

  writeFileSync(join(projectDir, '.env'), 'DATABASE_URL="postgresql://user:password@localhost:5432/mydb"\n')
  writeFileSync(join(projectDir, 'README.md'), `# ${name}\n\nSequelize ORM\n\nnpm run dev\n`)
}
