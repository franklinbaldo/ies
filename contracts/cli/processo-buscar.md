---
type: CommandContract
title: Buscar NUP em resposta de pesquisa Web capturada
operation: processo.buscar
command: ies processo NUP --resultado-pesquisa-file HTML_PRIVADO
effect: read
implementation: partial
source_commit: 50a0e41b26973d6adfb04a3cdd30c980ed8c345a
---

# Busca local de NUP fora da caixa em resposta HTML privada (Refs #58, #56)

`ies processo NUP --resultado-pesquisa-file HTML_PRIVADO [--json]` interpreta
**uma resposta da tabela de pesquisa Web já capturada** por operador
autorizado, sem navegar links e sem executar HTTP. A consulta não depende
da caixa da unidade nem da escolha de backend REST/SOAP.

## Entradas e invariantes

O NUP é `NumeroProcesso` opaco, com zeros preservados; caracteres de
controle são recusados antes de aparar espaços. Exigir um único NUP,
arquivo regular privado 0600 em Unix, tamanho máximo de 8 MB, sem symlink
no componente final (Linux). Recusar o modo combinado com `--resumo`,
`--historico-privado`, filtros de leitura integral, ações de escrita,
ou `--unidade` que sugira contexto live não demonstrado.

A resposta deve conter **uma única** tabela `pesquisaResultado`, com
linhas e links `procedimento_trabalhar` verificáveis, IDs internos
decimais distintos e NUPs únicos. Usar o parser Rust já ensaiado em
`pesquisa_web`, sem construir/abrir link assinado. Selecionar apenas
correspondência **exata** do NUP. Falhas distinguem número inválido,
arquivo privado, estrutura HTML e NUP não observado por erros e subcódigos
tipados; nenhuma classificação por mensagem.

## Saída e limite

JSON: `processo.numero`, `total_observado`,
`escopo: resposta_pesquisa_html_privado_offline`,
`autenticacao_remota: false`, `requisicoes: 0`.
Nunca exportar HTML, cookies, links, IDs internos ou infra_hash.
A ausência do NUP na captura é `numero_nao_observado`, **não**
`processo_inexistente`; não comprova busca integral ou atual.
A CLI `ies processo NUP` sem a opção continua stub; esta fatia
não executa POST de pesquisa nem obtém árvore/histórico do processo.

## Evidência e validação

- [Matriz Todos](../evidencias/processo-buscar.md): fluxos UnB, UFRJ,
  Goiás e PEN, sem assumir equivalência de versões/capturas.
- [Parser existente](../../src/pesquisa_web.rs) e
  [testes desta borda CLI](../../src/processo_pesquisa_privada_cli/tests.rs).
- CONTRACT → RED compilável antes de GREEN; validar offline pela CI.
- UnB/UFRJ/Goiás/PEN live: **NOT_RUN**. O operador deverá confirmar
  autorização/efeitos do POST antes de qualquer busca Web real.
