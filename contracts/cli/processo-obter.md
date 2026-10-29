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
o identificador interno do SEI. Substantivo sem NUP mostra ajuda.

O shim valida apenas a representação de entrada: vazio, caracteres de
controle e referências legadas `sei://` são rejeitados com erro tipado,
subcódigo estável e saída 2. Um NUP é uma cadeia opaca: não se exige
máscara, tamanho máximo arbitrário, apenas dígitos ou prefixo de órgão.
Cada instalação pode reconhecer outra numeração, inclusive de sistemas
externos. O valor preserva zeros à esquerda. Cabe ao SEI resolver se o
protocolo existe ou é acessível; a leitura permanece não implementada.

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

## Resolução de identidade (shim puro)

O resolvedor tipado compara primeiro o protocolo textual (sem diferença de
caixa) e só depois permite correspondência por letras e algarismos,
ignorando pontuação. Não altera o valor original, não escolhe o primeiro
entre candidatos ambíguos e não presume padrões locais. Retorna
`ErroResolucao::NaoEncontrado` ou `ErroResolucao::Ambiguo`, não
mensagens como discriminante. Para a caixa Web já interpretada, `resolver_na_caixa` seleciona a linha\ncom `IdProcedimento` preservado: duas aparições do mesmo ID nos grupos\nrecebidos e gerados não constituem ambiguidade; IDs distintos com\nprotocolos equivalentes constituem ambiguidade. Essa seleção não prova\na existência fora da caixa nem autoriza abrir ou receber processos.\n\nA resolução é um mecanismo reutilizável
para candidatos vindos de um adaptador autorizado; não representa
consulta ou descoberta de existência no SEI.

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
