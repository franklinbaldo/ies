---
type: Guide
title: Auditoria DRY transversal por Tree-sitter
description: Inventário AST de duplicações da codebase Rust e redução de serializações intermediárias na CLI
---

# Auditoria DRY — primeira passada transversal

Base: `7d44d84` de `main`. Execução via Python, `uvx`, Tree-sitter Rust
(`scripts/dry_ast.py`), sem requisições ao SEI.
Ensaio da AST: [GitHub Actions 42159813824](https://github.com/franklinbaldo/ies/actions/runs/42159813824).

## Cobertura e contagens da AST anterior à mudança

- 471 arquivos `.rs` de `src/` percorridos; 4 com AST parcial na gramática fixada (não alterados automaticamente).
- 1.088 funções inspecionadas acima do limiar de 45 tokens.
- 15 construções repetidas `serde_json::to_string_pretty(...).unwrap*` em `println!`, todas reconhecidas pelo codemod.
- 78 referências a `unwrap_or_default`, que exigem triagem semântica; não são automaticamente bugs.
- 666 chamadas `map_err`, muitas legítimas nas fronteiras de tipos; não alterar por contagem.
- 24 chamadas `Instalacao::interpretar` (candidatas à validação de sessão uma vez).
- 3 grupos de funções com corpos de tokens exatamente iguais, 21 grupos de estrutura lexical semelhante.

Os números são indicadores de revisão, não uma afirmação de que todos os
casos estejam incorretos. Reutilização correta de regras também é DRY.
Padrões de código de operações distintas podem ser semelhantes e ainda
assim exigir semântica, autorização ou verificação independentes.

## Transformação automática aplicada

O codemod percorreu todos os fontes e atualizou **15 emissores JSON**,
nos módulos da CLI de bloco, documento, processo e unidade.
Cada `println!("{}", serde_json::to_string_pretty(&valor).unwrap...)`
passou a emitir diretamente usando `serde_json::to_writer_pretty`
sobre o `stdout` bloqueado, seguido de quebra de linha. A apresentação
retorna erro ao ponto de chamada; não converte falha de saída em JSON vazio.
O helper `saida::imprimir_json<T: Serialize>` aceita valores tipados,
permitindo evitar futuramente `serde_json::Value` intermediário.

O `--json` continua sem cores, banners, tabelas ou sequências ANSI.
Rich (biblioteca a escolher em outra frente) é reservado à saída humana;
não há dependência nova nesta PR.

## Candidatos à segunda passada, sem fusão automática

1. Validação reiterada da origem em `processo_concluir/fluxo.rs`,
   `processo_reabrir/fluxo.rs`, `processo_enviar/fluxo.rs` e fluxos
   de documentos: preferir sessão tipada, sem dispensar autorização antes
   da rede.
2. Famílias de erros em que o mesmo erro é construído em vários pontos
   (issue #299), distinguindo nova causa de erro apenas propagado.
3. `transporte/resposta_http.rs`: `unwrap_or_default` sobre a leitura de
   trailers `chunked` pode ocultar truncamento; corrigir separadamente
   com testes de protocolo.
4. `unidade_trocar_cli.rs::aplicar_troca`: saída
   `Result<Option<usize>, &'static str>` descarta causas tipadas.
5. Uso da representação JSON como intermediário para apresentação
   humana, especialmente `processo_concluir_cli.rs` (issue #287).
6. `ErroHtml { onde: &'static str }` e mapeamentos de causa:
   identidade por ponto de emissão, sem strings como classificação
   interna; migração por domínio e testes de contrato.

## Limitações e segurança

- Tree-sitter identifica sintaxe, não prova equivalência semântica entre
  fluxos administrativos. Nenhum fluxo de escrita foi unificado.
- Clones lexicais não justificam criar um despachante genérico.
- Os quatro arquivos com AST parcial exigem análise com gramática mais
  recente compatível com os bindings; não houve reescrita neles.
- A mudança da CLI exige CI canônico: Rustfmt, Clippy, suíte Rust,
  golden de JSON, contratos e verificação da ausência de mudanças de
  contagem de HTTP.
- Nenhum acesso ao SEI foi executado durante esta auditoria.
