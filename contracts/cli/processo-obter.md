---
type: CommandContract
title: Ler processo administrativo
operation: processo.obter
command: ies processo NUP
effect: read
implementation: stub
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Ler processo administrativo

`ies processo NUP` consulta um processo pelo número visível, sem exigir
o identificador interno do SEI. Substantivo sem NUP mostra ajuda. Antes de consultar qualquer adaptador,\no shim de contrato valida o número com `NumeroProcesso`: vazio, mais de\n80 bytes ou caracteres de controle são recusados por variantes tipadas\nde `NumeroProcessoInvalido`, com `entrada_invalida`, subcódigo estável\ne saída 2. Identificadores válidos preservam zeros à esquerda.\nIsso ainda não implementa uma leitura do SEI.

## Entradas

- `NUP`: número do processo que o usuário pode consultar.
- `--sem-documentos`: omite a árvore de documentos.
- `--sem-marcadores`: omite os marcadores do processo.
- `--sem-andamentos`: omite os andamentos.
- `--sem-propriedades`: omite o cadastro e as propriedades complementares.
- `--andamentos-completos`: solicita a lista completa quando disponível.
- `--json` e `--unidade SIGLA`: opções globais.

## Saída prevista

O modelo de domínio `Processo` inclui o número exibido pela instalação,
documentos, marcadores, andamentos e propriedades, salvo projeções omitidas.
A URL estável, quando presente, não contém `infra_hash`.
Ausência de um dado não equivale a autorização para fabricar seu valor.

## Falhas e estado de implementação

O comando de domínio permanece não implementado até existir uma
fonte SEI configurada e com efeitos conhecidos. Evidências históricas
são reproduzidas separadamente pelo comando ies replay CENARIO.
Ao implementar, códigos 0/1/2/3 devem distinguir sucesso, falha,
uso inválido e efeito incerto; uma consulta não deve fabricar sessão.

## Evidência

[processo.obter](../evidencias/processo-obter.md) mapeia os cenários por
instalação. Veja também [árvore do processo](../../docs/como-o-sei-funciona/rotas-e-arvore.md)
e [paginação](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md).

A aprovação da interface não comprova paridade nem execução HTTP no IES.
