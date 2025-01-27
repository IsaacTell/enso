/** @file Container responsible for rendering and interactions in first half of forgot password
 * flow. */
import * as React from 'react'

import ArrowRightIcon from 'enso-assets/arrow_right.svg'
import AtIcon from 'enso-assets/at.svg'
import GoBackIcon from 'enso-assets/go_back.svg'

import * as app from '../../components/app'
import * as auth from '../providers/auth'

import Input from './input'
import Link from './link'
import SubmitButton from './submitButton'

// ======================
// === ForgotPassword ===
// ======================

/** A form for users to request for their password to be reset. */
export default function ForgotPassword() {
    const { forgotPassword } = auth.useAuth()

    const [email, setEmail] = React.useState('')

    return (
        <div className="flex flex-col gap-6 text-primary text-sm items-center justify-center min-h-screen">
            <form
                className="flex flex-col gap-6 bg-frame-selected rounded-2xl shadow-md p-8 w-full max-w-md"
                onSubmit={async event => {
                    event.preventDefault()
                    await forgotPassword(email)
                }}
            >
                <div className="font-medium self-center text-xl">Forgot Your Password?</div>
                <Input
                    required
                    validate
                    type="email"
                    autoComplete="email"
                    label="Email"
                    icon={AtIcon}
                    placeholder="Enter your email"
                    value={email}
                    setValue={setEmail}
                />
                <SubmitButton text="Send link" icon={ArrowRightIcon} />
            </form>
            <Link to={app.LOGIN_PATH} icon={GoBackIcon} text="Go back to login" />
        </div>
    )
}
