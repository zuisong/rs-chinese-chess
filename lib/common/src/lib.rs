/**
 * 详细中文注释 - 公共工具库（common）
 *
 * 目标
 * - 提供简单的公共函数集合，便于其他模块复用
 * - 本仓库中包含一个最小化的 add 示例函数及单元测试
 *
 * 使用注意
 * - 仅提供通用示例，实际功能应根据项目需要扩展
 */

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
