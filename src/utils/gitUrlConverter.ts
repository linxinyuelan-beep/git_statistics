/**
 * 将 Git 远程 URL 转换为 GitLab 提交页面 URL
 * 
 * 支持的输入格式:
 * 1. http://git.dev.sh.ctripcorp.com/global-rail-gds/gds-order-system.git
 * 2. git@git.dev.sh.ctripcorp.com:global-rail-gds/gds-order-system.git
 * 
 * 输出格式:
 * https://git.dev.sh.ctripcorp.com/global-rail-gds/gds-order-system/-/commit/{commitId}
 */

export function convertGitUrlToGitLabCommitUrl(remoteUrl: string, commitId: string): string | null {
  try {
    // 处理 SSH 格式: git@git.dev.sh.ctripcorp.com:global-rail-gds/gds-order-system.git
    if (remoteUrl.startsWith('git@')) {
      // 提取域名和路径
      const match = remoteUrl.match(/^git@([^:]+):(.+)$/);
      if (match && match[1] && match[2]) {
        const domain = match[1];
        let path = match[2];
        
        // 移除 .git 后缀
        if (path.endsWith('.git')) {
          path = path.slice(0, -4);
        }
        
        return `https://${domain}/${path}/-/commit/${commitId}`;
      }
    }
    
    // 处理 HTTP/HTTPS 格式: http://git.dev.sh.ctripcorp.com/global-rail-gds/gds-order-system.git
    if (remoteUrl.startsWith('http://') || remoteUrl.startsWith('https://')) {
      // 解析 URL
      const url = new URL(remoteUrl);
      let path = url.pathname;
      
      // 移除 .git 后缀
      if (path.endsWith('.git')) {
        path = path.slice(0, -4);
      }
      
      return `${url.protocol}//${url.host}${path}/-/commit/${commitId}`;
    }
    
    // 不支持的格式
    return null;
  } catch (error) {
    console.error('Error converting Git URL to GitLab commit URL:', error);
    return null;
  }
}