---
type: ImplementationDecision
title: Prévia segura de sobrestar e retomar pela barra Web
operation: processo.sobrestar
implementation: partial
status: adopted-for-scope
scope: unidade da sessão, identidade do processo e ações assinadas da árvore
selected_strategy: barra-assinada-situacao-verificada
alternatives: barra-assinada-situacao-verificada,rest,heuristica-pelo-texto
evidence_level: acervo-todos-e-testes-sinteticos
live_validation: not-run
---

# Decisão — sobrestar e retomar (#269)

A Web do SEI oferece `procedimento_sobrestar` como link da barra do
processo; retomar como `removerSobrestamentoProcesso();` apontando
à tela do nó. O status `AbertoEm::Sobrestado` na raiz distingue
o sobrestamento de unidades simplesmente fechadas.

Reaproveitam-se `barra_do_processo`, `endereco`, `IndiceLocal`
e `abrir_para_escrever`; o caminho de leitura ignora o link direto
da árvore para provar a unidade publicada pelo Controle. Nenhuma URL
assinada é construída, nenhum JavaScript executado, nenhum 302 seguido.
Os testes sintéticos impõem orçamento de três GET a frio, sem POST.

Alternativas descartadas: rota REST não observada; reconhecimento
por ícone/texto; assumir unidade sem `lnkInfraUnidade`;
construir por conta própria `procedimento_remover_sobrestamento`.

A prévia está funcional, mas `--confirmar` permanece fechado ANTES da
rede: faltam a conferência da forma exata do formulário servido e
a reconciliação pós-escrita para confirmar com segurança. A escrita
não pode ser considerada entregue com esta PR. A próxima fatia deve
implementar testes de POST/GET único, leitura de verificação,
efeito incerto (saída 3), replay do acervo e validação em treinamento
com registro de resíduos.

RED histórico desta PR: cinco testes falharam no CI inicial.
[Contrato](../../contracts/cli/processo-sobrestar.md),
[issue](https://github.com/franklinbaldo/ies/issues/269).
