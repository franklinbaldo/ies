---
type: AgentMemory
title: "Ordem da seleção é evidência de UX: 28 formulários confrontados"
description: "PR #539 confrontou 28 GETs de sete cassettes via anti-IES HTTP/SQLite; corrigiu perda da ordem original no menu e obteve CI 7/7."
agent: raposa
timestamp: "2026-11-09T05:33:41Z"
tags: [cli, ux, atribuicao, anti-ies, cassette, bdd, sqlite, ordenacao]
---

# Para retomar

**FEITO:** [PR #539](https://github.com/franklinbaldo/ies/pull/539) Ready, head `00d9a57d`, 0 commits atrás da main na última consulta. Estendi o gate `processo_atribuir::acervo::replay_de_processo_atribuir_no_acervo` para confrontar cada GET de formulário aceito das cassettes originais com o HTML GET servido pelo anti-IES em HTTP local, alimentado pelo SQLite efêmero com IDs/logins/rótulos extraídos da própria cassette. O mesmo leitor Rust interpreta ambos; compara identidade da ação, opções, **ordem** e rótulos. O NUP e a unidade usados como andaimes locais não vêm da cassette nem são tratados como evidência externa. O teste não habilita `--confirmar` da CLI.

**RED real:** [CI #42567076033](https://github.com/franklinbaldo/ies/actions/runs/42567076033) encontrou opção de atribuição fora da ordem observada em UnB: `atribuicoes::ler` ordenava pelo ID de usuário em vez da posição do `<option>` no documento. Corrigi a leitura SQLite para `ORDER BY rowid`, preservando a ordem de inserção transacional dos itens extraídos do menu. Não houve migração ou novo subcódigo. Ajustei o teste para não revelar logins/nomes em eventuais falhas de comparação.

**GREEN:** [CI oficial #42567676728](https://github.com/franklinbaldo/ies/actions/runs/42567676728) 7/7 PASS no SHA final, [rápido #42567668971](https://github.com/franklinbaldo/ies/actions/runs/42567668971) PASS. Gate de acervo relata **7 cassettes, 42 respostas classificadas, 28 formulários GET confrontados e aceitos**, sem acessar SEI externo. [Inspeção #42566749339](https://github.com/franklinbaldo/ies/actions/runs/42566749339) PASS de SHA-1/SHA-256 para as cassettes UnB/GO (24+26 interações); gatilho temporário removido. Handoff na [issue #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6733059300).

**Níveis de prova:** confronto semântico independente da **tela GET de formulário** agora PASS nas sete instalações. Prova anterior da **prévia pela CLI real ↔ HTTPS local ↔ anti-IES ↔ SQLite** segue PASS na main (#521/#536); não houve E2E de confirmação pública nem releitura de escrita CLI nesta fatia. POST histórico das cassettes não identifica autoria ou sucesso; treinamento/live e REA **NOT_RUN**. BDD-264-02/03 continuam abertos, e `--confirmar` continua fechado antes de sessão/rede. Nenhuma escrita remota, credencial ou produção.

**Cafezinho #388:** reli o debate da folha/contrafólio; uma coleção de quatro marcas preserva seus membros, mas pode esconder uma ordem relevante. No menu SEI, a ordem das opções também é uma observação, não apenas o conjunto de IDs. Esta analogia não constitui evidência externa sobre o SEI.

**CONTINUAR:** 🦅 revisar #539; 🐘/🦡 integrar. Em uma nova fatia, abordar confirmação da CLI real contra anti-IES/SQLite com POST único e releitura, depois ensaio autorizado em treinamento com prova independente, remoção e registro do resíduo. Nunca retry quando o efeito do POST é incerto.
