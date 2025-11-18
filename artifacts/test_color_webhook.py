#!/usr/bin/env python3
"""
飞书颜色组测试脚本 (Python 版本)
用于批量发送颜色测试卡片到飞书机器人
"""

import json
import sys
import time
import argparse
from pathlib import Path
from typing import Optional

try:
    import requests
except ImportError:
    print("❌ 需要安装 requests 库: pip install requests")
    sys.exit(1)


class ColorTester:
    """飞书颜色测试器"""

    def __init__(self, webhook_url: str, json_file: str = "artifacts/color_test_samples.json"):
        self.webhook_url = webhook_url
        self.json_file = Path(json_file)
        self.data = self._load_data()

    def _load_data(self) -> dict:
        """加载测试数据"""
        if not self.json_file.exists():
            raise FileNotFoundError(f"找不到测试文件: {self.json_file}")

        with open(self.json_file, 'r', encoding='utf-8') as f:
            return json.load(f)

    def send_color_test(self, group_id: int) -> bool:
        """发送单个颜色组测试"""
        if group_id < 0 or group_id >= len(self.data['samples']):
            print(f"❌ 无效的颜色组 ID: {group_id} (有效范围: 0-{len(self.data['samples'])-1})")
            return False

        sample = self.data['samples'][group_id]
        color_name = sample['color_name']
        card = sample['feishu_card']

        print(f"📤 发送颜色组 #{group_id}: {color_name}")

        try:
            response = requests.post(
                self.webhook_url,
                json=card,
                headers={'Content-Type': 'application/json'},
                timeout=10
            )

            result = response.json()
            status_code = result.get('code') or result.get('StatusCode') or response.status_code

            if status_code in [0, 200]:
                print(f"✅ 颜色组 #{group_id} 发送成功")
                return True
            else:
                print(f"❌ 颜色组 #{group_id} 发送失败: {result}")
                return False

        except requests.RequestException as e:
            print(f"❌ 网络错误: {e}")
            return False
        except Exception as e:
            print(f"❌ 未知错误: {e}")
            return False

    def send_all(self, delay: float = 1.0) -> int:
        """发送所有颜色组"""
        total = len(self.data['samples'])
        success_count = 0

        print(f"\n🎨 开始发送 {total} 组颜色测试卡片...")
        print(f"⏱️  发送间隔: {delay}s\n")

        for i in range(total):
            if self.send_color_test(i):
                success_count += 1

            if i < total - 1:
                time.sleep(delay)

        return success_count

    def send_range(self, start: int, end: int, delay: float = 1.0) -> int:
        """发送指定范围的颜色组"""
        success_count = 0
        total = end - start + 1

        print(f"\n🎨 发送颜色组 #{start} 到 #{end} (共 {total} 组)...")
        print(f"⏱️  发送间隔: {delay}s\n")

        for i in range(start, end + 1):
            if self.send_color_test(i):
                success_count += 1

            if i < end:
                time.sleep(delay)

        return success_count

    def list_colors(self):
        """列出所有颜色组"""
        print("\n🎨 可用的颜色组:\n")
        print("ID  | 颜色名称             | 背景色         | 文字色")
        print("-" * 65)

        for sample in self.data['samples']:
            gid = sample['group_id']
            name = sample['color_name']
            bg = sample['background']
            fg = sample['foreground']
            print(f"{gid:2d}  | {name:20s} | {bg:14s} | {fg}")

        print()


def main():
    parser = argparse.ArgumentParser(
        description='飞书颜色组测试工具',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  # 发送所有颜色组
  %(prog)s "https://open.feishu.cn/open-apis/bot/v2/hook/xxx"

  # 只发送第 3 组
  %(prog)s "https://open.feishu.cn/open-apis/bot/v2/hook/xxx" -s 3

  # 发送 0-5 组
  %(prog)s "https://open.feishu.cn/open-apis/bot/v2/hook/xxx" -r 0 5

  # 列出所有可用颜色组
  %(prog)s --list

  # 间隔 2 秒发送所有组
  %(prog)s "https://open.feishu.cn/open-apis/bot/v2/hook/xxx" -d 2
        """
    )

    parser.add_argument('webhook_url', nargs='?', help='飞书机器人 Webhook URL')
    parser.add_argument('-s', '--single', type=int, metavar='N',
                        help='只发送第 N 组颜色 (0-11)')
    parser.add_argument('-r', '--range', nargs=2, type=int, metavar=('START', 'END'),
                        help='发送指定范围的颜色组')
    parser.add_argument('-d', '--delay', type=float, default=1.0,
                        help='每次发送间隔秒数 (默认: 1.0)')
    parser.add_argument('-l', '--list', action='store_true',
                        help='列出所有可用的颜色组')
    parser.add_argument('-f', '--file', default='artifacts/color_test_samples.json',
                        help='测试数据文件路径')

    args = parser.parse_args()

    # 只列出颜色组
    if args.list:
        try:
            tester = ColorTester("dummy_url", args.file)
            tester.list_colors()
            return 0
        except Exception as e:
            print(f"❌ 错误: {e}")
            return 1

    # 需要 webhook_url
    if not args.webhook_url:
        parser.print_help()
        print("\n❌ 错误: 必须提供 Webhook URL")
        return 1

    try:
        tester = ColorTester(args.webhook_url, args.file)

        print(f"\n📡 Webhook URL: {args.webhook_url[:50]}...")

        # 单个颜色组
        if args.single is not None:
            success = tester.send_color_test(args.single)
            print(f"\n{'✅ 发送成功' if success else '❌ 发送失败'}")
            return 0 if success else 1

        # 范围颜色组
        elif args.range:
            start, end = args.range
            success_count = tester.send_range(start, end, args.delay)
            total = end - start + 1
            print(f"\n✅ 成功发送 {success_count}/{total} 组颜色测试")
            return 0 if success_count == total else 1

        # 所有颜色组
        else:
            success_count = tester.send_all(args.delay)
            total = len(tester.data['samples'])
            print(f"\n✅ 成功发送 {success_count}/{total} 组颜色测试")
            print("💡 请在飞书中查看显示效果")
            return 0 if success_count == total else 1

    except KeyboardInterrupt:
        print("\n\n⚠️  用户中断")
        return 130
    except Exception as e:
        print(f"\n❌ 错误: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
