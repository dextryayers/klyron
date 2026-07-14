import router from '@adonisjs/core/services/router'
import Server from '@adonisjs/core/services/server'

Server.errorHandler(() => import('@adonisjs/core/errors/handler'))
