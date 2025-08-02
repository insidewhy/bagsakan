#!/bin/bash

echo "Checking imports in your source files..."
echo ""

# Find all TypeScript files and look for imports from user-messaging-types
echo "Files importing from user-messaging-types:"
grep -r "from ['\"]\(.*user-messaging-types.*\)['\"]" --include="*.ts" --include="*.tsx" . 2>/dev/null | grep -v node_modules

echo ""
echo "Validator function calls found:"
grep -r "validate\(UserSawMessageThreadRequest\|SubscribeRequest\|SendMessageRequest\|GetMessageThreadRequest\|GetMessageThreadsRequest\)" --include="*.ts" --include="*.tsx" . 2>/dev/null | grep -v node_modules