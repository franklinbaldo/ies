---
type: SEIObservation
title: Inventário de efeitos das requisições Web
description: Declarações dos POSTs, evidência dos GETs e lacunas por destino
---

# Inventário de efeitos das requisições Web

Base examinada: `main@7a8b332`. Uma linha agrupa chamadas com o mesmo
papel, inclusive quando aparecem em mais de um comando. A evidência de
código identifica a requisição e a intenção do IES. Observações em outra
instalação ou no Todos ficam identificadas como tais. Não houve replay
nem validação ao vivo para este inventário.

## POSTs

| Requisição | Efeito avaliado no IES | Evidência |
| --- | --- | --- |
| Enviar credenciais SIP ou GO | `AlteraEstadoDaSessao` | [Descritor do login](../../src/sessao_login/tela.rs) e [telas de autenticação](sessao-login.md). |
| Escolher visão R ou D do Controle | `AlteraEstadoDaSessao` | [Dois descritores do Controle](../../src/visao_do_controle.rs). A próxima leitura pode herdar a visão. |
| Paginar recebidos ou gerados | `AlteraEstadoDaSessao` | [Descritor da paginação](../../src/pagina_do_controle.rs) e [observação das tabelas](tabelas-e-paginacao.md). |
| Pesquisar protocolo e paginar resultados | `AlteraEstadoDaSessao` nos dois envios | [Pesquisa](../../src/pesquisa_web/formulario_pesquisa.rs), [paginação](../../src/pesquisa_web/paginacao.rs) e [observação da tela](pesquisa-de-protocolo.md). |
| Pesquisar rapidamente, inclusive na resolução de processo ou documento | `AlteraEstadoDaSessao` no descritor do IES | [Descritor e abertura](../../src/processo_obter/abertura.rs). O [Todos @323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_previa_nao_envia_010.py) mede consulta na cassette `processo-excluir-pen1`. Consulta ao protocolo e mudança de contexto de sessão são efeitos distintos. |
| Alternar visão do histórico | `AlteraEstadoDaSessao` | [Descritor do histórico](../../src/historico_processo/visao.rs) e [observação da visão](historico-do-processo.md). |
| Salvar colunas do nível de detalhe | `AlteraPreferenciaDoUsuario` | [Descritor](../../src/nivel_de_detalhe/salvar.rs) e [releitura das marcas](efeitos-e-verificacao.md). |
| Concluir processo | `AlteraEstadoDoProcesso` | [Descritor](../../src/processo_concluir/tela.rs) e [observação da conclusão](conclusao-do-processo.md). |

A lista `LEITURAS_POR_POST` do Todos também contém autocompletes e
filtros que este snapshot do IES não emite. A presença da ação nessa
lista não basta para classificar outra forma do mesmo POST: a
cassette `documento-escritas-go`, por exemplo, distingue o recarregamento
do cadastro da gravação pelos campos enviados.

## GETs

| Requisição | Efeito e lacuna | Evidência |
| --- | --- | --- |
| Sondar raiz, abrir tela SIP/GO, abrir painel após autenticação | Efeito de sessão possível. Sem declaração por destino. | [Navegação do login](../../src/sessao_login/fluxo.rs) e [observação da sessão](sessao-login.md). |
| Abrir ou conferir Controle, inclusive após login e antes de buscar | Leitura de caixa e contexto de unidade. Sem declaração de efeito. | [Fluxo do login](../../src/sessao_login/fluxo.rs), [caixa](../../src/transporte.rs) e [tabelas](tabelas-e-paginacao.md). |
| Abrir formulário avançado de pesquisa | Leitura da tela. Sem declaração de efeito. | [Fluxo de pesquisa](../../src/pesquisa_web/fluxo.rs). |
| Abrir processo por link servido, pelo índice ou pela forma permitida por id, inclusive com documento | **Pode registrar recebimento** e alterar a visualização. O risco se aplica também à abertura usada na prévia ou na releitura de verificação. Sem declaração por destino. | [AGENTS](../../AGENTS.md), [abertura e releitura](../../src/processo_obter/abertura.rs). O [Todos @323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/docs/como-o-sei-funciona/07-verbo-http.md) registra a abertura e aponta visualização como expectativa sem verificação própria. |
| Abrir quadro da árvore e expandir pastas | Árvore sem declaração de efeito. Pastas têm declaração de mudança de sessão. | [Abertura](../../src/processo_obter/abertura.rs), [efeito de abrir pastas](../../src/processo_obter/pastas.rs) e [observação da árvore](arvore-do-processo.md). |
| Abrir histórico, propriedades ou marcadores do processo, inclusive por link guardado | Leitura de tela. Sem declaração nos GETs. | [Histórico](../../src/processo_obter/andamentos.rs), [propriedades](../../src/processo_obter/propriedades.rs) e [marcadores](../../src/processo_obter/marcadores.rs). |
| Abrir visualização e conteúdo de documento interno, lista de versões e conteúdo de versão selecionada | Leitura de documento. Sem declaração de efeito para esses destinos. | [Documento](../../src/processo_obter/documento.rs), [versões](../../src/processo_obter/documento_versao.rs) e [rotas da árvore](rotas-e-arvore.md). |
| Baixar documento externo | Leitura binária. Sem declaração de efeito remoto. | [Transporte binário](../../src/transporte/resposta_binaria.rs). |
| Abrir tela de nível de detalhe e reler depois de salvar | Leitura das marcas. Sem declaração de efeito para a abertura ou releitura. | [Fluxo de nível de detalhe](../../src/nivel_de_detalhe.rs) e [verificação](efeitos-e-verificacao.md). |
| Disparar reabertura por link servido e reler processo | O disparo altera o estado do processo. A releitura é outra abertura, com risco de registrar recebimento. | [Implementação](../../src/processo_reabrir.rs) e [observação da reabertura](reabertura-do-processo.md). |

## Representação proposta

Associar a cada destino GET um `EfeitoAvaliado` junto à regra que
confere sua URL, antes do transporte. Preservar o descritor que os
POSTs já usam. Usar `Desconhecido` enquanto faltar evidência e
reservar `ConsultaSemEfeitoObservado` para casos medidos com a evidência
identificada. A abertura de processo precisa distinguir o possível
registro de recebimento da simples leitura de HTML. Uma verificação
por releitura deve avaliar o efeito da própria abertura. A política
de autorização continua com cada operação.

A demonstração de lote sem efeito do
[Todos @323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_marcadores_lote_sem_efeito_010.py)
é referência para o tipo de evidência necessário. Ela não mede os
destinos deste inventário.
