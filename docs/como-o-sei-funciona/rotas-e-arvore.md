---
type: SEIObservation
title: Rotas e árvore do processo
description: Endpoints, frames e origem dos links assinados
---

# Rotas e árvore do processo

Páginas observadas utilizam `controlador.php?acao=...`; consultas
AJAX usam `controlador_ajax.php?acao_ajax=...` e podem responder XML.
A query costuma carregar `acao_origem`, `acao_retorno`,
`infra_sistema`, `infra_unidade_atual`, `id_procedimento`,
`id_documento`, `arvore` e `infra_hash`.

Cada página só contém os links que efetivamente publica.
A caixa disponibiliza o menu e os processos ali listados;
a pesquisa rápida pode localizar um processo fora da caixa.

A tela de trabalho pode carregar `ifrArvore` e
`ifrVisualizacao`. Na árvore, scripts podem declarar
`Nos[i]` e HTML das ações da barra do processo/documento.
O índice `i` não substitui a identidade de protocolo.

Árvores grandes podem distribuir documentos em pastas.
O script declara protocolos esperados; abrir a pasta pode
produzir outra resposta com nós. **A lista real precisa ser
conferida por identidade, não apenas pela contagem.**
Algumas versões oferecem abrir todas as pastas por um GET;
mesmo assim a integridade deve ser aferida.

A URL de uma ação pode aparecer como literal em função JS
ou como variável. Essas formas observadas não autorizam
execução genérica de JavaScript nem montagem especulativa de links.
