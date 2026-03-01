section .text
global _start

_start:
    ; 进入长模式
    call enter_long_mode
    
    ; 跳转到64位入口点
    jmp long_mode_start

; 进入长模式
enter_long_mode:
    ; 禁用中断
    cli
    
    ; 加载GDT
    lgdt [gdt64.pointer]
    
    ; 设置CR0寄存器，启用保护模式
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    ; 进入长模式准备
    mov eax, 10100000b
    mov cr4, eax
    
    ; 设置PAE页表
    mov eax, page_table_l4
    mov cr3, eax
    
    ; 启用长模式
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    
    ; 再次设置CR0，进入长模式
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    ret

; 64位入口点
long_mode_start:
    ; 清零所有段寄存器
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; 跳转到内核主函数或测试入口点
    %ifdef TEST
        extern test_main
        call test_main
    %else
        extern kernel_main
        call kernel_main
    %endif
    
    ; 无限循环
    hlt
    jmp $

; GDT定义
section .data
align 8
gdt64:
    dq 0 ; 空描述符
.code:
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ; 代码段
.data:
    dq (1 << 41) | (1 << 44) | (1 << 47) | (1 << 53) ; 数据段
.pointer:
    dw .pointer - gdt64 - 1
    dq gdt64

; 页表定义
align 4096
page_table_l4:
    dq page_table_l3 + 0b11
    times 511 dq 0

page_table_l3:
    dq page_table_l2 + 0b11
    times 511 dq 0

page_table_l2:
    dq page_table_l1 + 0b11
    times 511 dq 0

page_table_l1:
    %assign i 0
    %rep 512
        dq (i * 4096) + 0b10000011
        %assign i i + 1
    %endrep
