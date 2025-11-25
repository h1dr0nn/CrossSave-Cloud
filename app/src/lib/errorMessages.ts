/**
 * Converts technical error messages to user-friendly messages
 */
export function formatErrorMessage(error: unknown): string {
    if (typeof error === 'string') {
        return cleanErrorMessage(error);
    }

    if (error instanceof Error) {
        return cleanErrorMessage(error.message);
    }

    return 'An unexpected error occurred. Please try again';
}

/**
 * Cleans up error messages by removing technical prefixes and details
 */
function cleanErrorMessage(message: string): string {
    // Remove technical prefixes
    const prefixes = [
        'Network error: ',
        'Storage error: ',
        'IO error: ',
        'Serialization error: ',
        'Invalid configuration: ',
        'Unauthorized: ',
        'Not found: ',
    ];

    let cleaned = message;
    for (const prefix of prefixes) {
        if (cleaned.startsWith(prefix)) {
            cleaned = cleaned.substring(prefix.length);
            break;
        }
    }

    // Handle Tauri command validation errors
    if (cleaned.includes('invalid args') || cleaned.includes('missing required key')) {
        if (cleaned.includes('gameId')) {
            return 'Please enter a game ID';
        }
        if (cleaned.includes('versionId')) {
            return 'Please enter a version ID';
        }
        if (cleaned.includes('email')) {
            return 'Please enter your email';
        }
        if (cleaned.includes('password')) {
            return 'Please enter your password';
        }
        return 'Please fill in all required fields';
    }

    // Handle common error patterns
    if (cleaned.includes('error sending request') || cleaned.includes('Failed to fetch')) {
        return 'Unable to connect to cloud server. Please check your internet connection';
    }

    if (cleaned.includes('timeout') || cleaned.includes('timed out')) {
        return 'Connection timed out. Please try again';
    }

    if (cleaned.includes('status 409')) {
        return 'This email address is already registered';
    }

    if (cleaned.includes('status 401')) {
        return 'Invalid email or password';
    }

    if (cleaned.includes('status 404')) {
        return 'Account not found';
    }

    if (cleaned.includes('status 429')) {
        return 'Too many attempts. Please try again later';
    }

    if (cleaned.match(/status 5\d\d/)) {
        return 'Server error. Please try again later';
    }

    // Handle specific operation failures
    if (cleaned.includes('History list failed') || cleaned.includes('Failed to load history')) {
        return 'Unable to load history. Please check the game ID';
    }

    if (cleaned.includes('Rollback failed')) {
        return 'Unable to restore save. Please try again';
    }

    if (cleaned.includes('Delete failed')) {
        return 'Unable to delete. Please try again';
    }

    if (cleaned.includes('Package failed') || cleaned.includes('Manual package failed')) {
        return 'Unable to create save package. Please try again';
    }

    if (cleaned.includes('Watcher start failed')) {
        return 'Unable to start file watcher. Please check permissions';
    }

    if (cleaned.includes('Watcher stop failed')) {
        return 'Unable to stop file watcher';
    }

    if (cleaned.includes('Restore failed')) {
        return 'Unable to restore save. Please try again';
    }

    if (cleaned.includes('Failed to load profiles')) {
        return 'Unable to load emulator profiles';
    }

    if (cleaned.includes('Failed to save profile')) {
        return 'Unable to save profile. Please try again';
    }

    if (cleaned.includes('Failed to delete profile')) {
        return 'Unable to delete profile. Please try again';
    }

    if (cleaned.includes('Validation failed')) {
        return 'Validation failed. Please check your settings';
    }

    if (cleaned.includes('Failed to select directory')) {
        return 'Unable to select directory. Please try again';
    }

    // Return cleaned message or fallback
    return cleaned || 'An error occurred. Please try again';
}

/**
 * Formats validation error messages
 */
export function formatValidationError(field: string, message: string): string {
    const fieldNames: Record<string, string> = {
        email: 'Email',
        password: 'Password',
        device_id: 'Device ID',
        base_url: 'Server URL',
        api_key: 'API Key',
    };

    const friendlyField = fieldNames[field] || field;

    if (message.includes('required')) {
        return `${friendlyField} is required`;
    }

    if (message.includes('invalid')) {
        return `${friendlyField} is invalid`;
    }

    return message;
}
