---
type: ImplementationDecision
title: Criar processo pela escolha e cadastro publicados pelo SEI
operation: processo.criar
implementation: partial
status: adopted-for-scope
scope: escolha-de-tipo-e-cadastro-web-com-previa-e-verificacao-por-leitura
selected_strategy: web-formulario-servido-e-releitura-da-arvore
alternatives: web-formulario-servido-e-releitura-da-arvore,post-montado-estaticamente,rest-wssei
evidence_level: todos-e-captura-sei-409-sem-replay-ies
live_validation: not-run
---

# Decisão proposta: iniciar processo (issue #245)

## Alternativas consideradas

O oráculo Todos demonstra criação em instalações de treinamento.
A lista de tipos pode ser parcial; o cadastro e o modo de escolha
dependem da versão do SEI. O comando desejado é
`ies processo criar --tipo TIPO --especificacao TEXTO`,
com prévia por padrão, `--nivel publico|restrito`,
`--interessado` repetível e `--confirmar`.

1. **Web, formulário servido, confirmação por leitura** (proposta).
   Usa ações e controles observados, uma escrita e a árvore relida;
   compatível com o primitivo `formulario_web`.
2. POST montado de campos fixos: menos passos, porém incapaz
   de garantir que o tipo, a hipótese e os hidden sejam oferecidos.
3. REST/WSSEI: sem prioridade no IES e não demonstrado no oráculo.

## Decisão

Seguir a alternativa 1; a contagem RED original não foi medida e não deve ser inventada.
Escolher exclusivamente um tipo demonstrável pela lista completa,
abrir o cadastro pela ação publicada e conferir campos,
assunto sugerido, nível e hipótese antes de escrever.
Na prévia, apresentar o plano e os campos efetivamente lidos,
sem POST criador. O POST de escolha de tipo, quando publicado
pela versão, somente abre o cadastro.

Com `--confirmar`, o primitivo deve enviar uma única vez o
formulário derivado, com efeito avaliado de criação de processo.
Desabilitar redirecionamento automático; obter identidade da
resposta e reler a árvore. Falha posterior à escrita cuja leitura
não confirme a identidade é efeito incerto (3), sem repetição.
Não tratar simples HTTP 200/302 como prova suficiente.

Não usar regex nem executar JavaScript. Extrair atributos
unicamente por `src/html_atributos`; nunca expor URLs assinadas,
cookies ou `infra_hash`. A escrita live está autorizada **somente em treinamento**, conforme
comentário da issue #245 em 05/11/2026; a autorização não equivale à
validação da capacidade. Executar prévia, uma confirmação e releitura.

## Critérios antes da adoção

- RED: tipo ausente na lista completa, duplicado, incompleta,
  sem ação conhecida, assunto ausente/múltiplo, rádio desabilitado,
  hipótese obrigatória ausente, interessado ambíguo.
- GREEN: prévia com zero POST criador; confirmação com exatamente
  um POST criador e verificação fresca da identidade.
- Respostas: 302 com ID e NUP coincidentes, 200 com recusa,
  falha de rede depois do POST, 302 sem ID, árvore indisponível.
- Registrar contagem RED antes da implementação; replay de
  cassettes e orçamento por versão; executar gate pre-push e CI.
- Prévia live e escrita live têm registros independentes
  PASS/FAIL/NOT_RUN. Na ausência de execução: NOT_RUN.

## Comparação

A implementação Rust na PR #249 contém a leitura da lista,
conferência do cadastro, consulta AJAX de interessados, prévia Web,
POST de criação único, leitura da tela de trabalho e da árvore.
O formulário servido é a fonte dos campos e do destino do POST.
O teste com transporte injetado confirma o orçamento e o efeito
incerto sem retry. Ainda não há comprovação live por instalação,
nem replay integral do acervo das cassettes.
A Web mantém a prova dos campos; POST fixo e REST não oferecem
a mesma garantia de origem dos dados.

## Proteção de redirecionamentos e prova de RED — PR #283

A confirmação 4.0 simulada requer **6 requisições**: três GET da
prévia, um POST criador e dois GET de verificação. Na variante 4.1/5.0
testada, a escolha de tipo adiciona um POST **não criador**: são
**7 requisições**, dois POST de efeitos distintos e dois GET finais.
Os dois fluxos rejeitam efeito incerto sem repetir a escrita.

O parser rejeita `Location` com duas ocorrências de `acao` (mesmo
quando uma é esperada), com duas ocorrências de `id_procedimento`
(inclusive uma inválida) ou sem ID numérico único. A lista de tipos
4.0 igualmente rejeita ambiguidade em `acao` e
`id_tipo_procedimento` antes de abrir o cadastro.

Os dois testes de redirecionamento foram publicados antes da correção
no commit `043ad9b`. No CI
[42094129597](https://github.com/franklinbaldo/ies/actions/runs/42094129597),
**2 testes falharam** por aceitarem o redirecionamento ambíguo;
1392 passaram e 13 constavam como ignorados **preexistentes**.
Essa é evidência de RED **no CI**, não execução no sandbox local.
Os testes adicionados não foram desabilitados.

## Evidências verificáveis

- [Contrato parcial](../../contracts/cli/processo-criar.md)
- [Matriz de evidência](../../contracts/evidencias/processo-criar.md)
- [Teste Rust da superfície CLI](../../tests/cli_processo_criar.rs)
- [Observação indireta](../como-o-sei-funciona/inicio-do-processo.md)

## Limitações e revisão

A CLI foi conectada ao fluxo Web, mas a validação ao vivo continua
NOT_RUN. O CI testa o fluxo com transporte sintético e nunca escreve
no SEI real. A hipótese legal carregada dinamicamente por AJAX,
o replay integral do acervo e diferenças entre instalações
precisam de validação adicional. A escrita live foi autorizada na issue #245, exclusivamente em treinamento;
a prévia e a confirmação live continuam NOT_RUN enquanto não forem
executadas e documentadas. Reavaliar a escolha caso o replay mostre
cadastro não contemplado.

## Evidências e revisão

[Observação estrutural](../como-o-sei-funciona/inicio-do-processo.md);
[matriz do Todos](../../contracts/evidencias/processo-criar.md);
[issue #245](https://github.com/franklinbaldo/ies/issues/245).
Rever quando houver divergência no cadastro servido pela UFRJ/UnB
ou um replay que refute o redirecionamento observado pelo Todos.
