# Aurora DSQL Postgres.js React Sample

A React-based web application demonstrating how to connect to Amazon Aurora DSQL using the postgres.js connector in a browser environment.

## Features

- Browser-based SQL query execution
- WebSocket connection to Aurora DSQL
- Amazon Cognito authentication for secure access to IAM credentials
- React UI with query editor
- Real-time query results display 

## Prerequisites

- Node.js 16+ and npm
- An Aurora DSQL cluster
- AWS credentials with appropriate IAM permissions
- IAM role for Aurora DSQL access

## ⚠️ Important

* Running this code might result in charges to your AWS account.
* We recommend that you grant your code least privilege. At most, grant only the
  minimum permissions required to perform the task. For more information, see
  [Grant least privilege](https://docs.aws.amazon.com/IAM/latest/UserGuide/best-practices.html#grant-least-privilege).
* This code is not tested in every AWS Region. For more information, see
  [AWS Regional Services](https://aws.amazon.com/about-aws/global-infrastructure/regional-product-services).

## Amazon Cognito Setup

This sample uses Amazon Cognito to securely manage IAM credentials without embedding them in JavaScript code. Storing IAM credentials directly in client-side code is unsafe and not recommended for production applications. Amazon Cognito provides temporary, scoped credentials through user authentication, making it suitable for browser-based applications.

**Note:** This sample demonstrates secure credential management for a single-user scenario and is not intended to showcase multi-user query editor functionality.

### Create User Pool

A user pool is a user directory in Amazon Cognito that manages authentication, registration, and account recovery for users signing in with username/password. 

1. AWS Console > Amazon Cognito > User pools > Click on `Create user pool`
2. Application Type: Select `Single-page application SPA`
3. Name your application: `Aurora DSQL Query Editor React`
4. In Configure options under "Options for sign-in identifiers", check Email 
5. Under "Self-registration", uncheck `Enable self-registration`
6. Under "Required attributes for sign-up", select `email`
7. Return URL: http://localhost:3000 (note: it's http not https as the demo is run locally) 

### Add Allowed sign-out URL
1. AWS Console > User pools > Click on the newly created `User pool`
2. On the left side menu, click on Applications > App clients 
3. Select `Aurora DSQL Query Editor React`
4. Look for `Login Pages` tab above the `Quick setup guide` and click on it
5. Click on `Edit` button 
6. Under `Allowed sign-out URLs - optional`, add `http://localhost:3000`
  
### Create a user inside the user pool
1. Click on the user pool
2. User Management > Users 
3. Click on the `Create user` button 
4. Enter Email address 
5. Select `Mark email address as verified`
6. Set a temporary password (a new password will be set upon login)
7. Click on `Create User`

### Create Identity Pool

An identity pool provides temporary AWS credentials by federating authenticated users from identity providers like Amazon Cognito user pools, enabling secure access to AWS services.

1. AWS Console > Amazon Cognito > Identity pools
2. Click on `Create identity pool`
3. Under `User access`, select `Authenticated access`
4. Under `Authenticated identity sources`, select `Amazon Cognito user pool`
5. Click next
6. Use `Create a new IAM role` and enter `aurora_dsql_query_editor_sample_role` as the new role name 
7. Under `User pool details`, select the newly created user pool 
8. Under `App Client ID`, select the client id with `Aurora DSQL Query Editor React`
9. Click next 
10. Enter `aurora_dsql_sample_react_identity_pool` as the `Identity pool name`
11. Click next
12. Click `Create identity pool`

### Create a connect only policy
This policy allows the user to connect only, but doesn't allow cluster management (e.g. create/edit/delete clusters). 
1. Find out the resource ARN for your Aurora DSQL cluster 
2. AWS Console > Aurora DSQL > Clusters > select the cluster 
3. Under Cluster overview, the Amazon Resource Name for this cluster should be shown in this format `arn:aws:dsql:us-east-1:<ACCOUNT_NUMBER>:cluster/<CLUSTER_ID>`
4. AWS Console > Policies > Create policy
5. Select JSON 
6. CREATE a new policy with the following JSON:
```json
{
	"Version": "2012-10-17",
	"Statement": [
		{
			"Sid": "Statement1",
			"Effect": "Allow",
			"Action": [
				"dsql:DbConnect"
			],
			"Resource": "arn:aws:dsql:us-east-1:<ACCOUNT_NUMBER>:cluster/<CLUSTER_ID>"
		}
	]
}
```
7. Policy name: `aurora_dsql_db_connect_only`

### Attach policy to cognito role 
1. AWS Console > Roles > select `aurora_dsql_query_editor_sample_role`
2. Note down the ARN (shown in the format of `arn:aws:iam::<ACCOUNT_NUMBER>:role/service-role/aurora_dsql_query_editor_sample_role`) which will be used in the next section 
3. Click on `Add permissions` > `Attach policies`
4. Look for `aurora_dsql_db_connect_only` and select the check box on the left 
5. Click on `Add permissions`

### Register a new user and associate the IAM role 
1. Follow this guide [Authorizing database roles to connect to your cluster](https://docs.aws.amazon.com/aurora-dsql/latest/userguide/using-database-and-iam-roles.html#using-database-and-iam-roles-custom-database-roles) to create a new user `example` and associate your role ARN from step 2 above 
2. After this setup, the user `example` is ready to connect

## Configuration

Update the following values in `src/config.ts`:

### User Pool Settings

- `const region = "<COGNITO_REGION>"`;
  - Check the top right of the Amazon Cognito Console beside your AWS user name e.g. `us-west-2`
- `const userPoolId = "<USER_POOL_ID>";`
  - AWS Console > Amazon Cognito > User pools (left menu bar) > User pool ID
- `const clientId = "<USER_POOL_APP_CLIENT_ID>"; `
  - AWS Console >  Amazon Cognito > User pools (left menu bar) > Select your User Pool > App clients (left menu bar) 
- `const cognitoDomain = "<COGNITO_DOMAIN>";`
  - AWS Console >  Amazon Cognito > User pools (left menu bar) > Select your User Pool > App clients (left menu bar) > Select your App client > look for "cognitoDomain" under App.js sample 

### Identity Pool Settings
- `const identityPoolId = "<COGNITO_IDENTITY_POOL_ID>";`
  - AWS Console > Amazon Cognito > Identity pools (left menu bar) > Identity pool ID

### DSQL Cluster Settings
- `const dsqlHost = "your-cluster-endpoint.dsql.us-east-1.on.aws";`
  - AWS Console > Aurora DSQL > Clusters (left menu bar) > Endpoint

## Installation

```bash
npm install
```

## Running the Application

**Development mode:**
```bash
npm run dev
```

## Usage

1. Start the development server
2. Open http://localhost:3000 in your browser
3. Click "Login" and authenticate with your Cognito user credentials
4. Enter a SQL query in the textarea
5. Click "Execute Query" to run the query
6. View results in the output panel

## Project Structure

```
├── src/
│   ├── config.ts          # Centralized configuration (Cognito, Identity Pool, DSQL)
│   ├── App.tsx            # Main React component with authentication and query interface
│   └── index.tsx          # React entry point
├── public/
│   └── index.html         # HTML template
├── webpack.config.js      # Webpack configuration
├── webpack-perf-hooks.js  # Browser polyfill for Node.js perf_hooks
└── package.json           # Dependencies
```
## Browser Compatibility Configuration

Since postgres.js is designed for Node.js, several webpack configurations are required for browser compatibility:

### webpack.config.js

```javascript
resolve: {
  fallback: {
    "buffer": path.resolve(__dirname, 'node_modules/buffer/index.js'),
    "timers": path.resolve(__dirname, 'node_modules/timers-browserify/main.js'),
    "events": path.resolve(__dirname, 'node_modules/events/events.js'),
    "process/browser": path.resolve(__dirname, 'node_modules/process/browser.js'),
    "stream": false,
    "crypto": false,
    "fs": false,
    "path": false,
    "os": false,
    "tls": false,
    "net": false
  }
}
```
The above modules are replaced with browser equivalents or disabled. The perf_hooks module (Node.js high-resolution timer) requires an additional plugin.

```javascript
  new webpack.NormalModuleReplacementPlugin(
    /^perf_hooks$/,
    path.resolve(__dirname, 'webpack-perf-hooks.js')
  )
]
```

## License

This sample code is made available under the Apache-2.0 license. See the LICENSE file.
