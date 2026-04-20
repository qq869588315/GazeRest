import { render, screen } from '@testing-library/react'
import '../../i18n'
import { StatusPill } from './StatusPill'

describe('StatusPill', () => {
  it('renders translated status text', () => {
    render(<StatusPill status="paused" />)
    expect(screen.getByText('已暂停')).toBeInTheDocument()
  })
})
