---
type: CommandContract
title: Ler processo administrativo
operation: processo.obter
command: ies processo NUP
effect: read
implementation: partial
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
externos. O valor preserva zeros à esquerda. Cabe ao SEI resolver se o protocolo existe ou é acessível.
O recorte `--resumo` pode localizar o número **somente na caixa
inteira da unidade**, sem abrir o processo. A leitura integral permanece
não implementada.

## Entradas

- `NUP`: número do processo que o usuário pode consultar.
- `--resumo`: procura apenas a identidade já exibida na Caixa de Entrada,
  com um GET autorizado e sem acessar árvore/documentos/andamentos.
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

## Identidade e consulta

O NUP permanece opaco e é repassado ao mecanismo de consulta autorizado da
instalação. O IES não elimina pontuação para determinar equivalência, não
infere existência a partir da caixa da unidade e não inventa um resolvedor
de identidade. A resposta do SEI, quando esse contrato for implementado,
deverá fornecer a identidade do processo consultado; identificadores internos
são preservados quando efetivamente apresentados.

## Falhas e estado de implementação

Sem `--resumo`, a consulta integral continua não implementada. Com
`--resumo`, o comando lê a caixa da unidade uma vez, exige cobertura
dos totais declarados e retorna as ocorrências observadas agrupadas
como `recebidos` ou `gerados`. A ausência do número na caixa não
comprova inexistência do processo; produz `numero_nao_observado_na_caixa`.
Não há POST, abertura da árvore nem recebimento automático.
`--resumo` e `--andamentos-completos` são incompatíveis. Evidências históricas
são reproduzidas separadamente pelo comando ies replay CENARIO.
Ao implementar, códigos 0/1/2/3 devem distinguir sucesso, falha,
uso inválido e efeito incerto; uma consulta não deve fabricar sessão.

## Evidência

[processo.obter](../evidencias/processo-obter.md) mapeia os cenários por
instalação. Veja também [árvore do processo](../../docs/como-o-sei-funciona/rotas-e-arvore.md)
e [paginação](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md).

A aprovação da interface não comprova paridade nem execução HTTP no IES.


## Exportação privada do resumo observado na caixa (Refs #59)

`ies processo NUP --resumo --saida CAMINHO [--json]` grava **em novo arquivo
privado** a mesma projeção de identidade da caixa que `--resumo --json`
produz: número exato, unidade, categoria e ocorrências observadas.
`--saida` exige `--resumo`; o comando `processo NUP` sem resumo
não passa a consultar a árvore, documentos ou histórico.

O arquivo é criado sem sobrescrever destino preexistente, com permissão
0600 em Unix e erros tipados já definidos para arquivo da caixa. No
terminal, apenas estado e bytes gravados; nunca imprimir dados do resumo
no modo de exportação. Continua exigindo cobertura da caixa antes da
consulta e usa **o mesmo GET** autorizado da operação de resumo, sem
novas solicitações Web. Não equivale a processo completo, snapshot da
instalação ou consulta fora da caixa.
