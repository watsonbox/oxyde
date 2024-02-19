# frozen_string_literal: true

require_relative "oxyde/version"
require_relative "oxyde/oxyde"

module Oxyde
  class Error < StandardError; end

  class << self
    alias single_price_rust single_price
  end

  # Overwrite method in order to discard time zone (correct TZ for item is assumed)
  def self.single_price(identifier, starts_at, ends_at)
    single_price_rust(
      identifier,
      Time.new(starts_at.year, starts_at.month, starts_at.day, starts_at.hour, starts_at.min, 0, '+00:00').to_i,
      Time.new(ends_at.year, ends_at.month, ends_at.day, ends_at.hour, ends_at.min, 0, '+00:00').to_i
    )
  end
end
