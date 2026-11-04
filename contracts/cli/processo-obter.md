---
type: CommandContract
title: Ler processo administrativo
operation: processo.obter
command: ies processo NUP
effect: read-marks-viewed
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Ler processo administrativo

`ies processo NUP` consulta um processo pelo número visível, sem exigir
o identificador interno do SEI. Substantivo sem NUP mostra ajuda.

O shim valida apenas a representação de entrada: vazio, caracteres de
controle **em qualquer posição da entrada original, mesmo antes ou depois
de espaços** e referências legadas `sei://` são rejeitados com erro tipado,
subcódigo estável e saída 2. Um NUP é uma cadeia opaca: não se exige
máscara, tamanho máximo arbitrário, apenas dígitos ou prefixo de órgão.
Cada instalação pode reconhecer outra numeração, inclusive de sistemas
externos. O valor preserva zeros à esquerda. Cabe ao SEI resolver se o protocolo existe ou é acessível.
O comando abre o processo pela sessão de `ies login` e publica identidade e
documentos do primeiro nível da árvore
(seção "Leitura pela árvore"), os andamentos, as propriedades e os
marcadores (lista, um objeto por marcador;
[marcadores](processo-obter-marcadores.md)).

## Entradas

- `NUP`: número do processo que o usuário pode consultar.
- `--sem-documentos`: omite `documentos` e `pastas` (a árvore continua
  sendo lida, porque é ela que confirma a identidade;
  [pastas](processo-obter-pastas.md)).
- `--sem-marcadores`: não pede a tela de marcadores
  ([marcadores](processo-obter-marcadores.md)).
- `--sem-propriedades`: não pede a tela de propriedades
  ([propriedades](processo-obter-propriedades.md)).
- `--sem-andamentos`: não pede o histórico; `--andamentos-completos`: pede
  a visão completa ([andamentos](processo-obter-andamentos.md)). Juntos
  saem com 2.
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

Valem os subcódigos da seção "Leitura pela árvore". Evidências históricas
são reproduzidas separadamente pelo comando ies replay CENARIO.
Os códigos 0/1/2 distinguem sucesso, falha e uso inválido; uma consulta
não fabrica sessão.

## Leitura pela árvore (Refs #59, fatia 1)

O comando abre o processo pela sessão de `ies login`, com
o mínimo de requisições (link da árvore guardado, entrada por id,
pesquisa rápida guardada, Controle), lê o primeiro nível da árvore e
aprende no índice local. Resolução, orçamento por caminho (1 a 4
requisições), efeito declarado (`read-marks-viewed`: o GET da árvore
marca o processo como visualizado na unidade), saída JSON e subcódigos
estão em [abrir processo pela árvore](processo-obter-arvore.md).

## Evidência

[processo.obter](../evidencias/processo-obter.md) mapeia os cenários por
instalação. Veja também [árvore do processo](../../docs/como-o-sei-funciona/rotas-e-arvore.md)
e [paginação](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md).

A aprovação da interface não comprova paridade nem execução HTTP no IES.


## Opções aposentadas (decisão do proprietário, 04/11/2026)

`--resumo` e as opções que só existiam com ele ou com capturas privadas
— `--saida`, `--caixa-local`, `--observado-na-pagina`, `--pagina-arquivo`,
`--historico-privado` e `--resultado-pesquisa-file` — saíram de `ies
processo NUP`, sem alias: são erro de uso do Clap (saída 2, nada em
stdout, sem ler a sessão). Eram do regime anterior à página observada
(`--resumo` exigia cobertura de todas as páginas da caixa); a leitura pela
árvore com o índice local e a colheita do Controle cobrem o caso de uso
([decisão](../../docs/decisoes/processo-obter.md)). Capturas e cassettes
continuam lidas offline por `ies replay`.

## Identidade do título do histórico HTML (Refs #59)

Ao interpretar replay, o `title` do `head` define o processo se houver
cabeçalho. `title` de SVG no corpo não substitui a identidade. Dois
`head` ou dois títulos no cabeçalho falham com subcódigos distintos
`cabecalhos_html_do_historico_repetidos` e
`titulos_do_historico_repetidos`. Fragmentos legados sem `head` podem
ter um único título, apenas como compatibilidade offline. A operação
não faz HTTP, não comprova identidade de sessão nem processo integral.

## Normalização do NUP (Refs #59)

A entrada de `ies processo NUP` é convertida ao `NumeroProcesso` tipado
antes de qualquer leitura. Espaços comuns nas extremidades são removidos
uma única vez e os zeros do número permanecem exatos. CR/LF/tab continuam
recusados ainda na entrada original, com `numero_processo_controle`;
caracteres internos e pontuação não são transformados em equivalências
conjecturadas.

## Filtro de andamentos capturados: controles na entrada original (Refs #59)

Em `ies replay CENARIO --termo-andamento TERMO`, o filtro das descrições
verificadas deve recusar caracteres de controle na entrada **bruta**,
inclusive CR/LF/tab antes ou depois do texto. Eles não podem desaparecer
com `trim`: a variante Rust `ErroFiltroAndamentos::ContemControle`,
subcódigo `termo_de_andamento_contem_controle`, deve prevalecer sobre
`Vazio` quando houver controle. Espaços simples ao redor do termo
continuam tolerados, e termo inválido não pode descartar andamentos.
A busca continua restrita à cassette hash-verificada, sem HTTP nem
prova de histórico completo/live.
