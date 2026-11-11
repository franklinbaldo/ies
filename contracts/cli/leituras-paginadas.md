---
type: Contract
title: Leituras paginadas
description: Página observada por padrão e --pagina N nas leituras do Controle, parte do contrato da CLI
---

# Leituras paginadas

Parte do [contrato da CLI](../cli.md), separada por responsabilidade.

## Leituras paginadas: página observada por padrão (Refs #57)

Invariante de **toda** leitura paginada da CLI:

- **Sem opção**, a leitura usa só a tela já obtida (GET ou arquivo),
  sem POST de paginação, e devolve a página que a tela mostra com o
  `total_declarado` e a `faixa_observada` (`{"inicio": I, "fim": F}` das
  posições lidas; com zero registros a chave é omitida) de cada lista. A faixa não diz "primeira página": o servidor
  pode lembrar a última página pedida na sessão.
- A saída **nunca** publica `completa`, `completude` ou afirma lista
  integral ou snapshot; quem consome compara faixa e total.
- O opt-in é único e tem o mesmo nome em todos os comandos:
  **`--pagina N`** (inteiro ≥ 1), que pede exatamente a página N com
  o mínimo de requisições. Não há "todas as páginas".
- N fora do intervalo calculável pelo total e pelo tamanho de página
  observados é erro tipado **antes** do envio da página (se o tamanho da
  página só aparece depois de uma troca de visão, depois dela; a sessão
  fica na visão trocada). A página obtida confere unidade, visualização e
  critérios da tela de origem; não há retry. O efeito é
  `PaginaOuFiltroLembrado`: a leitura seguinte pode mostrar a página N, e a
  faixa publicada diz qual é. **Nenhuma leitura restaura** a visão ou a
  página que moveu (decisão do proprietário, 02/11/2026, na
  [decisão da operação](../../docs/decisoes/unidade-listar-processos.md)):
  com a sessão própria do IES, o efeito residual é cosmético (num SEI 5.x
  um login novo da mesma conta herda a última visão) e a página lembrada
  aparece em `faixa_observada` da leitura seguinte.

Contradições reais (faixa inválida ou não contígua, total ausente,
unidade ou contexto divergente) continuam recusas tipadas.

`documento DOC` lê documentos internos e externos pela Web (#60 fatias 1 a 3;
[contrato](documento-obter-web.md)). Continua como stub
`documento salvar --confirmar` remoto; `processo enviar`
([contrato](processo-enviar.md)) e `documento assinar`
([contrato](documento-assinar.md)) escrevem com prévia e verificação. `processo concluir` escreve com prévia e verificação
([contrato](processo-concluir.md)); `documento DOC --versao N` está
implementado ([contrato](documento-obter-versoes.md)). `processo NUP`
lê pela árvore ([contrato](processo-obter-arvore.md)) e `unidade
colunas --definir --confirmar` também escreve
([contrato](unidade-definir-colunas.md)). A prévia local de `documento salvar` não acessa o SEI. A autorização e a equivalência de outras rotas exigem
ensaios por cassette e por instalação.
