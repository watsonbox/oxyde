# frozen_string_literal: true

require 'date'

class PeriodPrice
  def inspect
    "<#{self.class.name} begin: #{self.begin}, price: #{price}>"
  end
end

RSpec.describe Oxyde do
  it 'has a version number' do
    expect(Oxyde::VERSION).not_to be nil
  end

  it 'calls the Rust extension' do
    expect(Oxyde.hello('World')).to eq('Hello from Rust, World!')
  end

  it 'builds the yield index' do
    expect(Oxyde.build_index).to be nil
  end

  it 'searches the yield index' do
    result = Oxyde.single_price(
      295_533,
      DateTime.parse('2024-02-08 12:00').to_time,
      DateTime.parse('2024-02-09 12:00').to_time
    )

    expect(result).to eq 65
  end
end
