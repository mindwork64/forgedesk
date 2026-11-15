pipeline {
    agent any
    options {
        timestamps()
        timeout(time: 40, unit: 'MINUTES')
        disableConcurrentBuilds(abortPrevious: true)
        buildDiscarder(logRotator(numToKeepStr: '10', artifactNumToKeepStr: '5'))
    }
    environment {
        CI = 'true'
    }
    stages {
        stage('Environment') {
            steps {
                sh '''
                set -eu
                echo "revision: $(git rev-parse HEAD)"
                node --version
                npm --version
                cargo --version
                rustc --version
                python3 --version
                '''
            }
        }
        stage('Verification') {
            steps {
                sh './scripts/ci.sh'
            }
        }
    }
    post {
        always {
            archiveArtifacts(
                artifacts: 'docs/ui/current/**',
                allowEmptyArchive: true,
                fingerprint: true
            )
        }
    }
}
