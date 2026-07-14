Rails.application.routes.draw do
  namespace :api do
    get 'health', to: 'health#show'
    post 'auth/login', to: 'auth#login'
    delete 'auth/logout', to: 'auth#logout'
  end
end
