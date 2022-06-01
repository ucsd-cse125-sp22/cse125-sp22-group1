import dynamic from 'next/dynamic'
import { Fragment, PropsWithChildren } from 'react'

const NoSsr = (props: PropsWithChildren<{}>) => (
	<Fragment>{props.children}</Fragment>
)

export default dynamic(() => Promise.resolve(NoSsr), {
	ssr: false
})