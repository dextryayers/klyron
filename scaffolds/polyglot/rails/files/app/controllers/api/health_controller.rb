module Api
  class HealthController < ApplicationController
    def show
      render json: { status: 'ok', service: '{{ name }}', version: '{{ version }}' }
    end
  end
end
